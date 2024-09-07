// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

pub struct XmlTag {
    name: String,
    attrs: String,
    content: String,
}

pub trait ToAttrs {
    fn to_attrs(&self) -> String;
}

impl<T: std::fmt::Display> ToAttrs for (&str, T) {
    fn to_attrs(&self) -> String {
        format!(
            " {}=\"{}\"",
            self.0,
            self.1
                .to_string()
                .replace("&", "&amp;")
                .replace("\"", "&quot;")
        )
    }
}

macro_rules! impl_to_attrs_for_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<$($t,)+> ToAttrs for ($($t,)+)
        where
            $($t: ToAttrs,)+
        {
            fn to_attrs(&self) -> String {
                "".to_owned()
                $(
                    + &self.$idx.to_attrs()
                )+
            }
        }
    };
}

impl_to_attrs_for_tuple!(0 A0);
impl_to_attrs_for_tuple!(0 A0, 1 A1);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5, 6 A6);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5, 6 A6, 7 A7);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5, 6 A6, 7 A7, 8 A8);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5, 6 A6, 7 A7, 8 A8, 9 A9);
impl_to_attrs_for_tuple!(0 A0, 1 A1, 2 A2, 3 A3, 4 A4, 5 A5, 6 A6, 7 A7, 8 A8, 9 A9, 10 A10);

impl XmlTag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            attrs: String::new(),
            content: String::new(),
        }
    }

    #[expect(dead_code)]
    pub fn add_text(&mut self, text: &str) -> &mut Self {
        self.content += &text
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;");
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.content += &text
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;");
        self
    }

    pub fn add_child(&mut self, child: Self) -> &mut Self {
        self.content += &child.to_string();
        self
    }

    pub fn with_child(mut self, child: Self) -> Self {
        self.content += &child.to_string();
        self
    }

    #[expect(dead_code)]
    pub fn add_attrs<A: ToAttrs>(&mut self, attrs: A) -> &mut Self {
        self.attrs += &attrs.to_attrs();
        self
    }

    pub fn with_attrs<A: ToAttrs>(mut self, attrs: A) -> Self {
        self.attrs += &attrs.to_attrs();
        self
    }
}

impl std::fmt::Display for XmlTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.content.is_empty() {
            write!(f, "<{}{}/>", self.name, self.attrs)
        } else {
            write!(
                f,
                "<{}{}>{}</{}>",
                self.name, self.attrs, self.content, self.name
            )
        }
    }
}

macro_rules! xml {
    ($tag:literal $(, $key:literal = $value:expr)* $(,)?) => (
        XmlTag::new($tag)
            $(
            .with_attrs(
                ($key, $value),
            )
            )*
    );
}

pub(crate) use xml;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn xml_macro() {
        assert_eq!(xml!("foo").to_string(), "<foo/>");
        assert_eq!(xml!("foo", "bar" = 1).to_string(), "<foo bar=\"1\"/>");
    }

    #[test]
    fn empty() {
        assert_eq!(XmlTag::new("test").to_string(), "<test/>");
    }

    #[test]
    fn with_text() {
        assert_eq!(
            XmlTag::new("test")
                .with_text("Hello, ")
                .with_text("<&username>")
                .to_string(),
            "<test>Hello, &lt;&amp;username&gt;</test>"
        );
    }

    #[test]
    fn with_attrs_one() {
        assert_eq!(
            XmlTag::new("test").with_attrs(("foo", 1)).to_string(),
            "<test foo=\"1\"/>"
        );
    }

    #[test]
    fn with_attrs_many() {
        assert_eq!(
            XmlTag::new("test")
                .with_attrs((("foo", 1), ("bar", &2), ("baz", "\"&\"")))
                .to_string(),
            "<test foo=\"1\" bar=\"2\" baz=\"&quot;&amp;&quot;\"/>"
        );
    }

    #[test]
    fn nested() {
        assert_eq!(
            XmlTag::new("html")
                .with_child(XmlTag::new("head").with_child(XmlTag::new("title").with_text("Hello")))
                .to_string(),
            "<html><head><title>Hello</title></head></html>"
        );
    }
}
