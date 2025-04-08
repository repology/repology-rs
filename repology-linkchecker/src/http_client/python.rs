// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

use anyhow::{Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

use crate::http_client::{HttpClient, HttpMethod, HttpRequest, HttpResponse};

const PYTHON_SOURCE: &str = include_str!("python/http_client.py");

#[derive(Serialize)]
struct PythonRequest {
    request_id: usize,
    url: String,
    use_get: bool,
    address: String,
    timeout: f32,
}

#[derive(Deserialize)]
struct PythonResponse {
    request_id: usize,
    response: HttpResponse,
}

#[derive(Default)]
struct State {
    next_request_id: usize,
    running_requests: HashMap<usize, oneshot::Sender<HttpResponse>>,
}

pub struct PythonHttpClient {
    request_tx: mpsc::Sender<(HttpRequest, oneshot::Sender<HttpResponse>)>,
}

impl PythonHttpClient {
    pub async fn new(user_agent: &str, python_path: &str) -> Result<Self> {
        let (started_tx, started_rx) = oneshot::channel::<bool>();

        let mut source_file = tempfile::NamedTempFile::new()?;
        source_file.write_all(PYTHON_SOURCE.as_bytes())?;
        let source_path = source_file.into_temp_path();

        info!("starting python process");

        let mut python_process = Command::new(python_path)
            .arg(&source_path)
            .arg(format!("--user-agent={user_agent}"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let python_stdin = python_process.stdin.take().expect("python stdin stream should be available because it was captured when constructing Command");
        let python_stdout = python_process.stdout.take().expect("python stdout stream should be available because it was captured when constructing Command");
        let python_stderr = python_process.stderr.take().expect("python stderr stream should be available because it was captured when constructing Command");

        let (request_tx, request_rx) =
            mpsc::channel::<(HttpRequest, oneshot::Sender<HttpResponse>)>(100);

        let state: Arc<Mutex<State>> = Default::default();

        {
            let state = state.clone();
            tokio::spawn(Self::handle_requests(state, python_stdin, request_rx));
        }
        tokio::spawn(Self::handle_responses(state, python_stdout, started_tx));
        tokio::spawn(Self::handle_messages(python_stderr));

        if started_rx.await? {
            info!("python process started");
        } else {
            bail!("failed to start python process");
        }

        Ok(Self { request_tx })
    }

    #[tracing::instrument(skip_all)]
    async fn handle_messages(python_stderr: ChildStderr) {
        let mut lines = BufReader::new(python_stderr).lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    warn!(line = line, "stderr message");
                }
                Ok(None) => {
                    info!("done on EOF");
                    break;
                }
                Err(error) => {
                    error!(error = %error, "read error");
                    break;
                }
            }
        }
    }

    #[tracing::instrument(skip_all)]
    async fn handle_requests(
        state: Arc<Mutex<State>>,
        mut python_stdin: ChildStdin,
        mut request_rx: mpsc::Receiver<(HttpRequest, oneshot::Sender<HttpResponse>)>,
    ) {
        loop {
            let Some((request, response_tx)) = request_rx.recv().await else {
                // happends when Checker is dropped → request_tx is dropped
                // leads to python_stdin closed → python process exiting on stdin EOF
                // (unless it's blocked in some way)
                info!("done");
                return;
            };

            let request = {
                let mut state = state.lock().unwrap();
                let request_id = state.next_request_id;
                state.next_request_id += 1;
                state.running_requests.insert(request_id, response_tx);

                PythonRequest {
                    request_id,
                    url: request.url,
                    use_get: match request.method {
                        HttpMethod::Get => true,
                        HttpMethod::Head => false,
                    },
                    address: format!("{}", request.address),
                    timeout: request.timeout.as_secs_f32(),
                }
            };

            python_stdin
                .write_all(
                    &serde_json::to_vec(&request)
                        .expect("should be able to serialize check request"),
                )
                .await
                .expect("should be able to write to python process stdin");
            python_stdin
                .write_all(b"\n")
                .await
                .expect("should be able to write to python process stdin");
            python_stdin
                .flush()
                .await
                .expect("should be able to write to python process stdin");
        }
    }

    #[tracing::instrument(skip_all)]
    async fn handle_responses(
        state: Arc<Mutex<State>>,
        python_stdout: ChildStdout,
        started_tx: oneshot::Sender<bool>,
    ) {
        let mut lines = BufReader::new(python_stdout).lines();

        match lines.next_line().await {
            Ok(Some(line)) if line == "python process started" => {
                started_tx
                    .send(true)
                    .expect("should be able to sent status flag to PythonHttpClient::new");
            }
            _ => {
                error!("failed to start python process");
                started_tx
                    .send(false)
                    .expect("should be able to sent status flag to PythonHttpClient::new");
                return;
            }
        }

        loop {
            let line = match lines.next_line().await {
                Ok(Some(line)) => line,
                Ok(None) => {
                    info!("done on EOF");
                    break;
                }
                Err(error) => {
                    error!(error = %error, "read error");
                    break;
                }
            };

            let response: PythonResponse =
                serde_json::from_str(&line).expect("must be able to deserialize check result");

            if let Some(response_tx) = state
                .lock()
                .unwrap()
                .running_requests
                .remove(&response.request_id)
            {
                response_tx
                    .send(response.response)
                    .expect("should be able to send response back to client");
            } else {
                error!(
                    request_id = response.request_id,
                    "got result for unknown request id"
                );
            }
        }

        let running_requests = &mut state.lock().unwrap().running_requests;

        if !running_requests.is_empty() {
            error!(
                num_requests = running_requests.len(),
                "remaining pending requests will be killed"
            );
            running_requests.clear();
        }
    }
}

#[async_trait]
impl HttpClient for PythonHttpClient {
    async fn request(&self, request: HttpRequest) -> HttpResponse {
        let (response_tx, response_rx) = oneshot::channel::<HttpResponse>();
        self.request_tx.send((request, response_tx)).await.expect("should be able to send requests (handle_requests task should be running as long as PythonHttpRequeser is alive)");
        response_rx.await.expect("should be able to receive response (handle_responses task should be running as long as PythonHttpRequeser is alive)")
    }
}
