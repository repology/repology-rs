// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fmt, fs, io};

use faine::inject_override_io_error;

const OLD_EXTENSION: &str = "old";
const NEW_EXTENSION: &str = "new";

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("filesystem operation failed")]
    Io(#[from] io::Error),
    #[error("operation cannot be performed because of concurrent transaction")]
    Busy,
}

#[derive(thiserror::Error)]
pub struct CommitError {
    error: Error,
    handle: WriteHandle,
}

impl CommitError {
    pub fn into_handle(self) -> WriteHandle {
        self.handle
    }
}

impl fmt::Debug for CommitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self.error, f)
    }
}

impl fmt::Display for CommitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.error, f)
    }
}

struct TransactionalDirInner {
    path: PathBuf,
    old_path: PathBuf,
    new_path: PathBuf,
    num_reading_handles: usize,
    num_writing_handles: usize,
}

#[derive(Clone)]
pub struct TransactionalDir {
    inner: Arc<Mutex<TransactionalDirInner>>,
}

impl TransactionalDir {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let old_path = path.with_added_extension(OLD_EXTENSION);
        let new_path = path.with_added_extension(NEW_EXTENSION);
        Self {
            inner: Arc::new(Mutex::new(TransactionalDirInner {
                path,
                old_path,
                new_path,
                num_reading_handles: 0,
                num_writing_handles: 0,
            })),
        }
    }

    pub fn is_clean(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        !inner.old_path.exists() && !inner.new_path.exists()
    }

    pub fn cleanup(&self) -> Result<(), io::Error> {
        let inner = self.inner.lock().unwrap();

        if !inner.path.exists() && inner.old_path.exists() {
            inject_override_io_error!(
                fs::rename(&inner.old_path, &inner.path),
                "cleanup: rename preserved old to current"
            )?;
        }
        if inner.old_path.exists() {
            inject_override_io_error!(
                fs::remove_dir_all(&inner.old_path),
                "cleanup: rename leftover old"
            )?;
        }
        if inner.new_path.exists() {
            inject_override_io_error!(
                fs::remove_dir_all(&inner.new_path),
                "cleanup: remove leftover new"
            )?;
        }
        Ok(())
    }

    pub fn current_state(&self) -> Option<ReadHandle> {
        let mut inner = self.inner.lock().unwrap();

        let path = if inner.path.exists() {
            inner.path.clone()
        } else if inner.old_path.exists() {
            inner.old_path.clone()
        } else {
            return None;
        };

        inner.num_reading_handles += 1;
        Some(ReadHandle {
            path,
            parent: Arc::clone(&self.inner),
        })
    }

    pub fn begin_replace(&self) -> Result<WriteHandle, Error> {
        let mut inner = self.inner.lock().unwrap();

        if inner.num_writing_handles > 0 {
            return Err(Error::Busy);
        }

        if inner.new_path.exists() {
            inject_override_io_error!(
                fs::remove_dir_all(&inner.new_path),
                "begin_replace: remove leftover new"
            )?;
        }
        inject_override_io_error!(
            fs::create_dir(&inner.new_path),
            "begin_replace: create empty new"
        )?;

        inner.num_writing_handles += 1;
        Ok(WriteHandle {
            path: inner.new_path.clone(),
            parent: Arc::clone(&self.inner),
        })
    }
}

pub struct ReadHandle {
    pub path: PathBuf,
    parent: Arc<Mutex<TransactionalDirInner>>,
}

impl Drop for ReadHandle {
    fn drop(&mut self) {
        let mut inner = self.parent.lock().unwrap();

        assert!(inner.num_reading_handles > 0);
        inner.num_reading_handles -= 1;
    }
}

pub struct WriteHandle {
    pub path: PathBuf,
    parent: Arc<Mutex<TransactionalDirInner>>,
}

impl WriteHandle {
    pub fn commit(self) -> Result<(), CommitError> {
        let parent = Arc::clone(&self.parent);
        let mut inner = parent.lock().unwrap();

        if inner.num_reading_handles > 0 {
            drop(inner);
            return Err(CommitError {
                error: Error::Busy,
                handle: self,
            });
        }
        assert!(inner.num_writing_handles > 0);

        if inner.path.exists() {
            if inner.old_path.exists()
                && let Err(e) = inject_override_io_error!(
                    fs::remove_dir_all(&inner.old_path),
                    "commit: remove leftover new"
                )
            {
                drop(inner);
                return Err(CommitError {
                    error: Error::Io(e),
                    handle: self,
                });
            }
            if let Err(e) = inject_override_io_error!(
                fs::rename(&inner.path, &inner.old_path),
                "commit: rename current to old"
            ) {
                drop(inner);
                return Err(CommitError {
                    error: Error::Io(e),
                    handle: self,
                });
            }
        }
        if let Err(e) = inject_override_io_error!(
            fs::rename(&inner.new_path, &inner.path),
            "commit: rename new to current"
        ) {
            drop(inner);
            return Err(CommitError {
                error: Error::Io(e),
                handle: self,
            });
        }
        // commit is deemed successful here
        inner.num_writing_handles -= 1;
        std::mem::forget(self);

        if inner.old_path.exists() {
            // does not fail a transaction
            let _ = inject_override_io_error!(
                fs::remove_dir_all(&inner.old_path),
                "commit: remove preserved old"
            );
        }
        Ok(())
    }
}

impl Drop for WriteHandle {
    fn drop(&mut self) {
        let mut inner = self.parent.lock().unwrap();

        assert!(inner.num_writing_handles > 0);
        let _ = inject_override_io_error!(fs::remove_dir_all(&inner.new_path), "abort: remove new");
        inner.num_writing_handles -= 1;
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use std::path::Path;

    fn write_to_dir(path: &Path, name: &str) {
        fs::File::create(&path.join(name)).unwrap();
    }

    fn read_dir(path: &Path) -> Vec<String> {
        let mut res: Vec<String> = fs::read_dir(path)
            .unwrap()
            .map(|res| res.unwrap().file_name().to_str().unwrap().to_owned())
            .collect();
        res.sort();
        res
    }

    #[test]
    fn test_main() {
        let tmpdir = tempfile::tempdir().unwrap();
        let dir = TransactionalDir::new(tmpdir.path().join("data"));

        // initially clean, no state
        assert!(dir.is_clean());
        assert!(dir.current_state().is_none());

        {
            // start new state, initially empty
            let tx = dir.begin_replace().unwrap();
            assert!(read_dir(&tx.path).is_empty());

            // add file to new state and commit
            write_to_dir(&tx.path, "aaa");
            assert!(tx.commit().is_ok());

            assert!(dir.is_clean());
        }

        {
            // check committed state
            let cur = dir.current_state().unwrap();
            assert_eq!(read_dir(&cur.path), vec!["aaa".to_string()]);
        }

        {
            // start another state
            let tx = dir.begin_replace().unwrap();
            assert!(read_dir(&tx.path).is_empty());

            write_to_dir(&tx.path, "bbb");

            {
                // current state still does not see any of this
                let cur = dir.current_state().unwrap();
                assert_eq!(read_dir(&cur.path), vec!["aaa".to_string()]);
            }

            assert!(tx.commit().is_ok());
        }

        // current state now sees the change
        let cur = dir.current_state().unwrap();
        assert_eq!(read_dir(&cur.path), vec!["bbb".to_string()]);
    }

    #[test]
    fn test_read_write_conflict() {
        let tmpdir = tempfile::tempdir().unwrap();
        let dir = TransactionalDir::new(tmpdir.path().join("data"));

        dir.begin_replace().unwrap().commit().unwrap();

        let r = dir.current_state().unwrap();
        let w = dir.begin_replace().unwrap();

        // try to commit, while read handle is still present
        let res = w.commit();
        assert!(res.is_err());
        let res = res.inspect_err(|err| {
            assert_eq!(format!("{:?}", err), "Busy".to_string());
            assert_eq!(
                format!("{}", err.to_string()),
                "operation cannot be performed because of concurrent transaction".to_string()
            );
        });

        // get write handle back from the error
        let w = res.unwrap_err().into_handle();

        drop(r);

        // second commit succeeds
        assert!(w.commit().is_ok());
    }

    #[test]
    fn test_write_write_conflict() {
        let tmpdir = tempfile::tempdir().unwrap();
        let dir = TransactionalDir::new(tmpdir.path().join("data"));

        let w = dir.begin_replace().unwrap();

        // try to open second write transaction
        let res = dir.begin_replace();
        let _ = res.inspect_err(|err| {
            assert_eq!(format!("{:?}", err), "Busy".to_string());
            assert_eq!(
                format!("{}", err.to_string()),
                "operation cannot be performed because of concurrent transaction".to_string()
            );
        });

        // first transaction is still functional
        assert!(w.commit().is_ok());
    }

    #[test]
    fn test_faine_full_coverage() {
        let report = faine::Runner::default()
            .run(|_trace| {
                let tmpdir = tempfile::tempdir().unwrap();
                let dir = TransactionalDir::new(tmpdir.path().join("data"));

                let mut expected_contents = vec![];

                // We need three transactions for 100% coverage, covering this rare scenario:
                // - First transaction succeeds, creating state
                // - Second transaction succeeds, but fails to cleanup preserved old state
                // - Third transaction also fails to cleanup leaftover old state from the previous tx
                for contents in ["first", "second", "third"] {
                    if let Ok(handle) = dir.begin_replace() {
                        write_to_dir(&handle.path, contents);

                        if !faine::inject_override!(false, "forced abort", true) {
                            let mut res = handle.commit();
                            if let Err(err) = res {
                                res = err.into_handle().commit();
                            }
                            if res.is_ok() {
                                expected_contents = vec![contents];
                            }
                            // XXX: Needs faine support https://github.com/AMDmi3/faine/issues/20
                            //assert_eq!(res.is_ok(), trace.trace().failpoint_status_last("commit: rename new to current") == Some(faine::Branch::Skip));
                            //if trace.trace().failpoint_status_last("commit: remove preserved old") == Some(faine::Branch::Activate)) {
                            //  assert_eq!(!dir.is_clean());
                            //}
                        }
                    };

                    if faine::inject_override!(false, "forced cleanup", true) {
                        if dir.cleanup().is_ok() {
                            assert!(dir.is_clean());
                        }
                    }

                    assert_eq!(
                        dir.current_state()
                            .map(|current_state| read_dir(&current_state.path))
                            .unwrap_or_default(),
                        expected_contents
                    );
                }

                // fourth transaction always succeeds
                faine::enable_failpoints!(false);

                {
                    let handle = dir.begin_replace().unwrap();
                    write_to_dir(&handle.path, "fourth");
                    handle.commit().unwrap();
                }

                assert_eq!(
                    read_dir(&dir.current_state().unwrap().path),
                    vec!["fourth".to_string()]
                )
            })
            .unwrap();

        eprintln!("{:#?}\n{} trace(s)", report.traces, report.traces.len());
    }
}
