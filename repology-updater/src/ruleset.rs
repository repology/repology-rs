// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Deserializer};
use walkdir::WalkDir;

fn from_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a sequence of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                vec.push(value);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(Visitor)
}

#[derive(Debug)]
enum FlavorAction {
    Auto,
    Explicit(Vec<String>),
}

impl<'de> Deserialize<'de> for FlavorAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FlavorAction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("true, a string, or a sequence of strings")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value {
                    Ok(FlavorAction::Auto)
                } else {
                    Err(E::custom(
                        "false not allowed as addflavor/setflavor argument".to_string(),
                    ))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(FlavorAction::Explicit(vec![value.to_owned()]))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(value) = seq.next_element::<String>()? {
                    vec.push(value);
                }
                Ok(FlavorAction::Explicit(vec))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Rule {
    // checks
    #[serde(deserialize_with = "from_string_or_vec")]
    name: Vec<String>,
    namepat: Option<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    ver: Vec<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    notver: Vec<String>,
    verpat: Option<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    wwwpart: Vec<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    sourceforge: Vec<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    summpart: Vec<String>,
    wwwpat: Option<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    ruleset: Vec<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    noruleset: Vec<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    category: Vec<String>,
    categorypat: Option<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    maintainer: Vec<String>,
    vercomps: Option<u32>,
    verlonger: Option<u32>,
    vergt: Option<String>,
    verge: Option<String>,
    verlt: Option<String>,
    verle: Option<String>,
    vereq: Option<String>,
    verne: Option<String>,
    relgt: Option<String>,
    relge: Option<String>,
    rellt: Option<String>,
    relle: Option<String>,
    releq: Option<String>,
    relne: Option<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    flag: Vec<String>,
    #[serde(deserialize_with = "from_string_or_vec")]
    noflag: Vec<String>,
    is_p_is_patch: Option<bool>,
    hasbranch: Option<bool>,

    // actions
    setname: Option<String>,
    setver: Option<String>,
    addflavor: Option<FlavorAction>,
    setflavor: Option<FlavorAction>,
    resetflavors: Option<bool>,
    setbranch: Option<String>,
    setbranchcomps: Option<u32>,
    remove: Option<bool>,
    ignore: Option<bool>,
    devel: Option<bool>,
    weak_devel: Option<bool>,
    altver: Option<bool>,
    altscheme: Option<bool>,
    vulnerable: Option<bool>,
    p_is_patch: Option<bool>,
    any_is_patch: Option<bool>,
    sink: Option<bool>,
    outdated: Option<bool>,
    legacy: Option<bool>,
    nolegacy: Option<bool>,
    incorrect: Option<bool>,
    untrusted: Option<bool>,
    noscheme: Option<bool>,
    rolling: Option<bool>,
    snapshot: Option<bool>,
    successor: Option<bool>,
    debianism: Option<bool>,
    generated: Option<bool>,
    trace: Option<bool>,
    #[serde(deserialize_with = "from_string_or_vec")]
    addflag: Vec<String>,
    last: Option<bool>,
    tolowername: Option<bool>,
    replaceinname: Option<HashMap<String, String>>,
    warning: Option<String>,
    setsubrepo: Option<String>,
    maintenance: Option<bool>,
    disposable: Option<bool>,
    precious: Option<bool>,
}

impl Rule {
    pub fn to_value(&self) -> anyhow::Result<serde_json::value::Value> {
        let mut fields = serde_json::Map::new();

        fn add_vec(
            map: &mut serde_json::Map<String, serde_json::value::Value>,
            name: &str,
            values: &[String],
        ) {
            if values.len() == 1 {
                map.insert(name.to_string(), values[0].clone().into());
            } else if values.len() > 1 {
                map.insert(name.to_string(), values.into());
            }
        }
        fn add_str(
            map: &mut serde_json::Map<String, serde_json::value::Value>,
            name: &str,
            value: &Option<String>,
        ) {
            if let Some(value) = value {
                map.insert(name.to_string(), value.clone().into());
            }
        }
        fn add_int(
            map: &mut serde_json::Map<String, serde_json::value::Value>,
            name: &str,
            value: &Option<u32>,
        ) {
            if let Some(value) = *value {
                map.insert(name.to_string(), value.into());
            }
        }
        fn add_bool(
            map: &mut serde_json::Map<String, serde_json::value::Value>,
            name: &str,
            value: &Option<bool>,
        ) {
            if let Some(value) = *value {
                map.insert(name.to_string(), value.into());
            }
        }
        fn add_fa(
            map: &mut serde_json::Map<String, serde_json::value::Value>,
            name: &str,
            value: &Option<FlavorAction>,
        ) {
            if let Some(value) = value {
                map.insert(
                    name.to_string(),
                    match value {
                        FlavorAction::Auto => true.into(),
                        FlavorAction::Explicit(vec) if vec.len() == 1 => vec[0].clone().into(),
                        FlavorAction::Explicit(vec) => vec.clone().into(),
                    },
                );
            }
        }

        add_vec(&mut fields, "name", &self.name);
        add_str(&mut fields, "namepat", &self.namepat);
        add_vec(&mut fields, "ver", &self.ver);
        add_vec(&mut fields, "notver", &self.notver);
        add_str(&mut fields, "verpat", &self.verpat);
        add_vec(&mut fields, "wwwpart", &self.wwwpart);
        add_vec(&mut fields, "sourceforge", &self.sourceforge);
        add_vec(&mut fields, "summpart", &self.summpart);
        add_str(&mut fields, "wwwpat", &self.wwwpat);
        add_vec(&mut fields, "ruleset", &self.ruleset);
        add_vec(&mut fields, "noruleset", &self.noruleset);
        add_vec(&mut fields, "category", &self.category);
        add_str(&mut fields, "categorypat", &self.categorypat);
        add_vec(&mut fields, "maintainer", &self.maintainer);
        add_int(&mut fields, "vercomps", &self.vercomps);
        add_int(&mut fields, "verlonger", &self.verlonger);
        add_str(&mut fields, "vergt", &self.vergt);
        add_str(&mut fields, "verge", &self.verge);
        add_str(&mut fields, "verlt", &self.verlt);
        add_str(&mut fields, "verle", &self.verle);
        add_str(&mut fields, "vereq", &self.vereq);
        add_str(&mut fields, "verne", &self.verne);
        add_str(&mut fields, "relgt", &self.relgt);
        add_str(&mut fields, "relge", &self.relge);
        add_str(&mut fields, "rellt", &self.rellt);
        add_str(&mut fields, "relle", &self.relle);
        add_str(&mut fields, "releq", &self.releq);
        add_str(&mut fields, "relne", &self.relne);
        add_vec(&mut fields, "flag", &self.flag);
        add_vec(&mut fields, "noflag", &self.noflag);
        add_bool(&mut fields, "is_p_is_patch", &self.is_p_is_patch);
        add_bool(&mut fields, "hasbranch", &self.hasbranch);

        add_str(&mut fields, "setname", &self.setname);
        add_str(&mut fields, "setver", &self.setver);
        add_fa(&mut fields, "addflavor", &self.addflavor);
        add_fa(&mut fields, "setflavor", &self.setflavor);
        add_bool(&mut fields, "resetflavors", &self.resetflavors);
        add_str(&mut fields, "setbranch", &self.setbranch);
        add_int(&mut fields, "setbranchcomps", &self.setbranchcomps);
        add_bool(&mut fields, "remove", &self.remove);
        add_bool(&mut fields, "ignore", &self.ignore);
        add_bool(&mut fields, "devel", &self.devel);
        add_bool(&mut fields, "weak_devel", &self.weak_devel);
        add_bool(&mut fields, "altver", &self.altver);
        add_bool(&mut fields, "altscheme", &self.altscheme);
        add_bool(&mut fields, "vulnerable", &self.vulnerable);
        add_bool(&mut fields, "p_is_patch", &self.p_is_patch);
        add_bool(&mut fields, "any_is_patch", &self.any_is_patch);
        add_bool(&mut fields, "sink", &self.sink);
        add_bool(&mut fields, "outdated", &self.outdated);
        add_bool(&mut fields, "legacy", &self.legacy);
        add_bool(&mut fields, "nolegacy", &self.nolegacy);
        add_bool(&mut fields, "incorrect", &self.incorrect);
        add_bool(&mut fields, "untrusted", &self.untrusted);
        add_bool(&mut fields, "noscheme", &self.noscheme);
        add_bool(&mut fields, "rolling", &self.rolling);
        add_bool(&mut fields, "snapshot", &self.snapshot);
        add_bool(&mut fields, "successor", &self.successor);
        add_bool(&mut fields, "debianism", &self.debianism);
        add_bool(&mut fields, "generated", &self.generated);
        add_bool(&mut fields, "trace", &self.trace);
        add_vec(&mut fields, "addflag", &self.addflag);
        add_bool(&mut fields, "last", &self.last);
        add_bool(&mut fields, "tolowername", &self.tolowername);
        if let Some(replaceinname) = &self.replaceinname {
            fields.insert(
                "replaceinname".to_string(),
                replaceinname.clone().into_iter().collect(),
            );
        }
        add_str(&mut fields, "warning", &self.warning);
        add_str(&mut fields, "setsubrepo", &self.setsubrepo);
        add_bool(&mut fields, "maintenance", &self.maintenance);
        add_bool(&mut fields, "disposable", &self.disposable);
        add_bool(&mut fields, "precious", &self.precious);

        Ok(serde_json::value::Value::Object(fields))
    }

    pub fn to_yaml(&self) -> anyhow::Result<String> {
        Ok(serde_saphyr::to_string(&serde_saphyr::FlowMap(
            self.to_value()?,
        ))?)
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Ruleset {
    pub rules: Vec<Rule>,
}

impl Ruleset {
    pub fn parse(path: &Path) -> anyhow::Result<Self> {
        let mut rules: Vec<Rule> = vec![];

        for entry in WalkDir::new(path).sort_by_file_name().into_iter() {
            let entry = entry?;
            if entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|filename| filename.ends_with(".yaml"))
            {
                let mut chunk: Vec<Rule> =
                    serde_saphyr::from_str(&std::fs::read_to_string(entry.path())?)
                        .with_context(|| format!("while parsing {}", entry.path().display()))?;
                rules.append(&mut chunk);
            }
        }

        Ok(Self { rules })
    }

    pub fn to_yaml(&self) -> anyhow::Result<String> {
        let mut out = String::new();
        for rule in &self.rules {
            out += "- ";
            out += &rule.to_yaml()?;
        }
        Ok(out)
    }
}
