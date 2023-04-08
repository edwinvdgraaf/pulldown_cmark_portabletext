pub mod portabletext {
    use core::panic;
    use std::io;

    use pulldown_cmark::Event::*;
    use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    #[cfg(feature = "serde_serialization")]
    use serde::{Serialize, Serializer};

    #[cfg(feature = "serde_serialization")]
    impl Serialize for Decorators {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                Decorators::LinkReference(d) => serializer.serialize_str(d),
                Decorators::AssetReference(a) => serializer.serialize_str(a),
                Decorators::Emphasis => serializer.serialize_str("em"),
                Decorators::Strong => serializer.serialize_str("strong"),
                Decorators::Strike => serializer.serialize_str("strike"),
                Decorators::Underline => serializer.serialize_str("underline"),
                Decorators::Code => serializer.serialize_str("code"),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    pub struct Source {
        pub srcset: String,
        pub width: u32,
        pub height: u32,
        #[cfg_attr(feature = "serde_serialization", serde(rename = "type"))]
        pub attr_type: String,
        pub media: Option<String>,
    }

    #[derive(Debug, PartialEq, Clone)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    pub struct Picture {
        pub src: String,
        pub alt: String,
        pub width: u32,
        pub height: u32,
        pub sources: Vec<Source>,
    }

    pub trait AssetReferenceResolver {
        fn resolve(&self, reference: String) -> String;
        fn resolve_picture(&self, reference: String, alt: String) -> Picture;
    }

    struct IdentityResolver;

    impl AssetReferenceResolver for IdentityResolver {
        fn resolve(&self, reference: String) -> String {
            reference
        }

        fn resolve_picture(&self, reference: String, alt: String) -> Picture {
            Picture {
                src: reference,
                alt,
                width: 999,
                height: 999,
                sources: vec![],
            }
        }
    }

    pub struct Options<'a> {
        pub asset_resolver: &'a dyn AssetReferenceResolver,
    }

    impl Default for Options<'_> {
        fn default() -> Self {
            Self {
                asset_resolver: &IdentityResolver {},
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    #[cfg_attr(feature = "serde_serialization", serde(rename_all = "camelCase"))]
    pub struct MarkDefLink {
        #[cfg_attr(feature = "serde_serialization", serde(rename = "_key"))]
        pub _key: String,
        #[cfg_attr(feature = "serde_serialization", serde(rename = "_type"))]
        pub _type: String,
        pub href: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    #[cfg_attr(feature = "serde_serialization", serde(rename_all = "camelCase"))]
    pub struct MarkDefImage {
        #[cfg_attr(feature = "serde_serialization", serde(rename = "_key"))]
        pub _key: String,
        #[cfg_attr(feature = "serde_serialization", serde(rename = "_type"))]
        pub _type: String,
        pub src: String,
        pub picture: Picture,
        #[cfg_attr(
            feature = "serde_serialization",
            serde(skip_serializing_if = "Option::is_none")
        )]
        pub caption: Option<String>,
    }

    #[derive(Debug, PartialEq, Clone)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    #[cfg_attr(feature = "serde_serialization", serde(rename_all = "camelCase"))]
    #[cfg_attr(feature = "serde_serialization", serde(untagged))]
    pub enum MarkDef {
        Link(MarkDefLink),
        Image(MarkDefImage),
    }

    #[derive(Debug, PartialEq, Clone)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    #[cfg_attr(feature = "serde_serialization", serde(rename_all = "camelCase"))]
    pub struct Asset {
        #[cfg_attr(feature = "serde_serialization", serde(rename = "_ref"))]
        pub _ref: String,
        pub src: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum Decorators {
        Emphasis,
        Strong,
        Strike,
        Underline,
        Code,
        LinkReference(String),
        AssetReference(String),
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    #[cfg_attr(feature = "serde_serialization", serde(rename_all = "lowercase"))]
    pub enum ListItemType {
        Bullit,
        Numbered,
    }

    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    #[cfg_attr(feature = "serde_serialization", serde(rename_all = "camelCase"))]
    pub struct SpanNode {
        #[cfg_attr(feature = "serde_serialization", serde(rename = "_type"))]
        pub _type: String,
        pub text: String,
        pub marks: Vec<Decorators>,
    }

    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde_serialization", derive(Serialize))]
    #[cfg_attr(feature = "serde_serialization", serde(rename_all = "camelCase"))]
    pub struct BlockNode {
        #[cfg_attr(feature = "serde_serialization", serde(rename = "_type"))]
        pub _type: String,
        #[cfg_attr(
            feature = "serde_serialization",
            serde(skip_serializing_if = "Option::is_none")
        )]
        pub style: Option<String>,

        #[cfg_attr(
            feature = "serde_serialization",
            serde(skip_serializing_if = "Vec::is_empty")
        )]
        pub children: Vec<SpanNode>,

        // meta on marks
        pub mark_defs: Vec<MarkDef>,
        // list items
        #[cfg_attr(
            feature = "serde_serialization",
            serde(skip_serializing_if = "Option::is_none")
        )]
        pub level: Option<usize>,
        #[cfg_attr(
            feature = "serde_serialization",
            serde(skip_serializing_if = "Option::is_none")
        )]
        pub list_item: Option<ListItemType>,

        #[cfg_attr(
            feature = "serde_serialization",
            serde(skip_serializing_if = "Option::is_none")
        )]
        pub language: Option<String>,

        #[cfg_attr(
            feature = "serde_serialization",
            serde(skip_serializing_if = "Option::is_none")
        )]
        pub code: Option<String>,
    }

    // TODO: split this into multiple types
    impl BlockNode {
        pub fn default(style: String) -> Self {
            Self {
                _type: "block".to_string(),
                style: Some(style),
                children: vec![],
                mark_defs: vec![],
                language: None,
                code: None,
                level: None,
                list_item: None,
            }
        }

        pub fn code(language: String, code: String) -> Self {
            Self {
                _type: "code".to_string(),
                style: None,
                children: vec![],
                mark_defs: vec![],
                language: Some(language),
                code: Some(code),
                level: None,
                list_item: None,
            }
        }

        pub fn default_list_item(level: usize, list_item: ListItemType) -> Self {
            Self {
                _type: "block".to_string(),
                style: Some("normal".to_string()),
                level: Some(level),
                list_item: Some(list_item),
                children: Vec::with_capacity(2),
                mark_defs: vec![],
                language: None,
                code: None,
            }
        }

        pub fn with_children(mut self, children: Vec<SpanNode>) -> Self {
            self.children = children;
            self
        }
    }

    struct PortabletextWriter<'a, I> {
        iter: I,
        writer: &'a mut Vec<BlockNode>,
        open_block: bool,
        active_list_item: Vec<ListItemType>,
        list_item_level: usize,
        active_markers: Vec<Decorators>,
        options: Options<'a>,
    }
    impl<'a, I> PortabletextWriter<'a, I>
    where
        I: Iterator<Item = Event<'a>>,
    {
        fn new(iter: I, writer: &'a mut Vec<BlockNode>, options: Options<'a>) -> Self {
            Self {
                iter,
                writer,
                options,
                open_block: false,
                active_markers: Vec::with_capacity(3),
                active_list_item: Vec::with_capacity(5),
                list_item_level: 0,
            }
        }

        // Writes a buffer, and tracks whether or not a newline was written.
        #[inline]
        fn write(&mut self, s: BlockNode) -> io::Result<()> {
            // dont think there are much worse places then this to put this but ohh well...
            self.open_block = true;
            self.writer.push(s);
            Ok(())
        }

        pub fn run(mut self) -> io::Result<()> {
            while let Some(event) = self.iter.next() {
                match event {
                    Start(tag) => {
                        self.start_tag(tag)?;
                    }
                    End(tag) => {
                        self.end_tag(tag)?;
                    }
                    Text(text) => {
                        let no_marks = self.active_markers.to_vec().is_empty();
                        if let Some(last_span) = self.last_span() {
                            if last_span.marks.is_empty() && no_marks {
                                self.add_text(text)?;
                            } else {
                                self.add_span(text)?;
                            }
                        } else {
                            self.add_span(text)?;
                        }
                    }
                    Code(code) => {
                        self.mark_start(Decorators::Code)?;
                        self.add_span(code)?;
                        self.mark_stop(Decorators::Code)?;
                    }
                    SoftBreak => {
                        if let Some(last_span) = self.last_span() {
                            last_span.text += " ";
                        }
                    }
                    Html(_) | FootnoteReference(_) | Rule | HardBreak | TaskListMarker(_) => {}
                }
            }
            Ok(())
        }

        fn consume_inner(&mut self) -> String {
            let mut nest = 0;
            let mut buffer = String::new();
            for event in &mut self.iter {
                match event {
                    Start(_) => nest += 1,
                    End(_) => {
                        if nest == 0 {
                            break;
                        }
                        nest -= 1;
                    }
                    Html(text) | Code(text) | Text(text) => {
                        buffer.push_str(&text);
                    }
                    SoftBreak | HardBreak | Rule => {
                        buffer.push(' ');
                    }
                    _ => {}
                }
            }
            buffer.to_owned()
        }

        /// Writes the start of an HTML tag.
        fn start_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
            match tag {
                Tag::Paragraph => {
                    if !self.open_block {
                        self.write(BlockNode::default("normal".to_string()))
                    } else {
                        Ok(())
                    }
                }
                Tag::BlockQuote => self.write(BlockNode::default("blockquote".to_string())),
                Tag::CodeBlock(CodeBlockKind::Fenced(syntax)) => {
                    let code = self.consume_inner();
                    self.write(BlockNode::code(syntax.into_string(), code))
                }
                Tag::CodeBlock(CodeBlockKind::Indented) => {
                    let plain_text: String = "plain_text".to_owned();
                    let code = self.consume_inner();
                    self.write(BlockNode::code(plain_text, code))
                }
                Tag::Heading(level) => {
                    let styling = format!("h{}", level);
                    self.write(BlockNode::default(styling))
                }
                Tag::List(options) => {
                    if options.is_some() {
                        self.active_list_item.push(ListItemType::Numbered);
                    } else {
                        self.active_list_item.push(ListItemType::Bullit);
                    }
                    self.list_item_level += 1;
                    Ok(())
                }
                Tag::Item => {
                    let list_item = *self.active_list_item.last().unwrap();
                    let level = self.list_item_level;
                    self.write(BlockNode::default_list_item(level, list_item))
                }
                Tag::Link(_link_type, link_href, _link_title) => {
                    let key: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(12)
                        .map(char::from)
                        .collect();
                    let mark_def = MarkDef::Link(MarkDefLink {
                        _type: "link".to_owned(),
                        _key: key.to_owned(),
                        href: link_href.to_string(),
                    });

                    self.add_mark_def(mark_def).unwrap();
                    self.mark_start(Decorators::LinkReference(key))
                }
                Tag::Image(_image_type, image_href, title) => {
                    let key: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(12)
                        .map(char::from)
                        .collect();

                    let alt = self.consume_inner();
                    let src = self.options.asset_resolver.resolve(image_href.to_string());
                    let picture = self
                        .options
                        .asset_resolver
                        .resolve_picture(image_href.to_string(), alt.to_owned());

                    if let Some(last_block) = self.last_block() {
                        let caption = if !title.is_empty() {
                            Some(title.to_string())
                        } else {
                            None
                        };

                        last_block.mark_defs.push(MarkDef::Image(MarkDefImage {
                            _key: key.to_owned(),
                            _type: "image".to_owned(),
                            src,
                            picture,
                            caption,
                        }));

                        self.mark_start(Decorators::AssetReference(key.to_owned()))?;
                        self.add_span_with_type(alt.into(), "figure".to_owned())?;
                        // No end tag emited for image so stop marker
                        return self.mark_stop(Decorators::AssetReference(key));
                    }

                    Ok(())
                }
                Tag::Strong => self.mark_start(Decorators::Strong),
                Tag::Emphasis => self.mark_start(Decorators::Emphasis),
                Tag::Strikethrough => self.mark_start(Decorators::Strike),

                Tag::FootnoteDefinition(_)
                | Tag::Table(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell => Ok(()),
            }
        }

        fn end_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
            match tag {
                Tag::Strong => self.mark_stop(Decorators::Strong),
                Tag::Emphasis => self.mark_stop(Decorators::Emphasis),
                Tag::Strikethrough => self.mark_stop(Decorators::Strike),
                Tag::Link(_link_type, link_url, _link_title) => {
                    let mark_def = self
                        .last_block()
                        .unwrap()
                        .mark_defs
                        .iter()
                        .find(|d| {
                            if let MarkDef::Link(l) = d {
                                l.href == link_url.to_string()
                            } else {
                                false
                            }
                        })
                        .unwrap_or_else(|| panic!("mark def missing for {}", link_url));

                    let key = match mark_def {
                        MarkDef::Link(l) => l._key.as_str().to_owned(),
                        _ => panic!("Cannot find key for link"),
                    };

                    self.mark_stop(Decorators::LinkReference(key))
                }
                Tag::List(_options) => {
                    self.active_list_item.pop();
                    self.list_item_level -= 1;
                    Ok(())
                }
                _ => self.close_block(),
            }
        }

        fn add_text(&mut self, text: CowStr<'a>) -> io::Result<()> {
            if let Some(last_span) = self.last_span() {
                last_span.text += &text;
            }
            Ok(())
        }
        fn add_span(&mut self, text: CowStr<'a>) -> io::Result<()> {
            self.add_span_with_type(text, "span".to_owned())
        }

        fn add_span_with_type(&mut self, text: CowStr<'a>, _type: String) -> io::Result<()> {
            let marks: Vec<Decorators> = self.active_markers.to_vec();
            if let Some(current_node) = self.last_block() {
                current_node.children.push(SpanNode {
                    _type,
                    text: text.to_string(),
                    marks,
                });
            }

            Ok(())
        }

        fn close_block(&mut self) -> io::Result<()> {
            self.open_block = false;
            Ok(())
        }

        fn last_block(&mut self) -> Option<&mut BlockNode> {
            let length = self.writer.len();
            self.writer.get_mut(length - 1)
        }

        fn last_span(&mut self) -> Option<&mut SpanNode> {
            self.last_block().and_then(|last_block| {
                let last_block_children_length = last_block.children.len();

                if last_block_children_length == 0 {
                    return None;
                }

                last_block.children.get_mut(last_block_children_length - 1)
            })
        }

        fn add_mark_def(&mut self, mark_def: MarkDef) -> io::Result<()> {
            if let Some(last_block) = self.last_block() {
                last_block.mark_defs.push(mark_def)
            }
            Ok(())
        }

        fn mark_start(&mut self, decorator: Decorators) -> io::Result<()> {
            self.active_markers.push(decorator);
            Ok(())
        }

        fn mark_stop(&mut self, decorator: Decorators) -> io::Result<()> {
            let index = self.active_markers.iter().position(|d| d == &decorator);
            self.active_markers.remove(index.unwrap());
            Ok(())
        }
    }

    pub fn push_portabletext<'a, I>(output: &'a mut Vec<BlockNode>, parser: I)
    where
        I: Iterator<Item = Event<'a>>,
    {
        push_portabletext_with_opts(output, parser, Options::default())
    }

    pub fn push_portabletext_with_opts<'a, I>(
        output: &'a mut Vec<BlockNode>,
        parser: I,
        options: Options<'a>,
    ) where
        I: Iterator<Item = Event<'a>>,
    {
        PortabletextWriter::new(parser, output, options)
            .run()
            .unwrap();
    }
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

#[cfg(test)]
mod tests {
    use crate::portabletext::{self, MarkDef};
    use crate::portabletext::{BlockNode, Decorators, ListItemType, SpanNode};
    use pulldown_cmark::Parser;
    #[cfg(feature = "serde_serialization")]
    use serde_json;
    #[test]
    fn it_supports_heading() {
        let markdown_input = "# Hey";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("h1".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "Hey".to_string(),
            marks: vec![],
        }]);
        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn it_supports_headings() {
        let markdown_input = "# Hey \n ## HeyHey";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("h1".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "Hey".to_string(),
            marks: vec![],
        }]);

        let second_node = BlockNode::default("h2".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "HeyHey".to_string(),
            marks: vec![],
        }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
        assert_eq!(&second_node, portabletext_output.get(1).unwrap());
    }

    #[test]
    fn it_works_heading_three() {
        let markdown_input = "### Hey";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("h3".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "Hey".to_string(),
            marks: vec![],
        }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn works_for_multiple_blocks() {
        let markdown_input = r#"
Experiment with cazy stuff, but get the basics right. But what are these basics, 
that is an interesting question. What for one can feel like such a no brainer, can be in another eyes seem like a total waste. 

## All endings with beginings

> Like the legend of the phoenix
> All ends with beginnings
> What keeps the planet spinning (uh)
> The force of love beginning
>
> _Pharrel Williams_
        "#;

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        assert_eq!(
            &BlockNode::default("h2".to_string()).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "All endings with beginings".to_string(),
                marks: vec![],
            }]),
            portabletext_output.get(1).unwrap()
        );
        assert_eq!(4, portabletext_output.len());
    }

    #[test]
    fn it_works_heading_three_with_bold() {
        let markdown_input = "### Hey __strong__";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("h3".to_string()).with_children(vec![
            SpanNode {
                _type: "span".to_string(),
                text: "Hey ".to_string(),
                marks: vec![],
            },
            SpanNode {
                _type: "span".to_string(),
                text: "strong".to_string(),
                marks: vec![Decorators::Strong],
            },
        ]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn bold_block() {
        let markdown_input = "__strong__";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("normal".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "strong".to_string(),
            marks: vec![Decorators::Strong],
        }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn support_newline_in_blocks() {
        let markdown_input = "new line can have multiple\nnewlines";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("normal".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "new line can have multiple newlines".to_string(),
            marks: vec![],
        }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn support_newlines_in_blocks_with_emphasis() {
        let markdown_input = "new line can have multiple\n*newlines*";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("normal".to_string()).with_children(vec![
            SpanNode {
                _type: "span".to_string(),
                text: "new line can have multiple ".to_string(),
                marks: vec![],
            },
            SpanNode {
                _type: "span".to_string(),
                text: "newlines".to_string(),
                marks: vec![Decorators::Emphasis],
            },
        ]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn nested_marking() {
        let markdown_input = "__strong *emp*__";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("normal".to_string()).with_children(vec![
            SpanNode {
                _type: "span".to_string(),
                text: "strong ".to_string(),
                marks: vec![Decorators::Strong],
            },
            SpanNode {
                _type: "span".to_string(),
                text: "emp".to_string(),
                marks: vec![Decorators::Strong, Decorators::Emphasis],
            },
        ]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn blockquotes() {
        let markdown_input = "> Okay, pep talk!\n\n Hi there";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node =
            BlockNode::default("blockquote".to_string()).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Okay, pep talk!".to_string(),
                marks: vec![],
            }]);

        let second_node = BlockNode::default("normal".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "Hi there".to_string(),
            marks: vec![],
        }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
        assert_eq!(&second_node, portabletext_output.get(1).unwrap());
    }

    #[test]
    fn blockquotes_multiline() {
        let markdown_input = "> Okay, pep talk!\n Hi there";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node =
            BlockNode::default("blockquote".to_string()).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Okay, pep talk! Hi there".to_string(),
                marks: vec![],
            }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn lists_unordered() {
        let markdown_input = "* Item 1\n  * Item 1.1\n * Item 2";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node =
            BlockNode::default_list_item(1, ListItemType::Bullit).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 1".to_string(),
                marks: vec![],
            }]);

        let second_node =
            BlockNode::default_list_item(2, ListItemType::Bullit).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 1.1".to_string(),
                marks: vec![],
            }]);

        let third_node =
            BlockNode::default_list_item(1, ListItemType::Bullit).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 2".to_string(),
                marks: vec![],
            }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
        assert_eq!(&second_node, portabletext_output.get(1).unwrap());
        assert_eq!(&third_node, portabletext_output.get(2).unwrap());
    }

    #[test]
    fn lists_ordered() {
        let markdown_input = "1. Item 1\n    1. Item 1.1\n 2. Item 2";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node =
            BlockNode::default_list_item(1, ListItemType::Numbered).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 1".to_string(),
                marks: vec![],
            }]);

        let second_node =
            BlockNode::default_list_item(2, ListItemType::Numbered).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 1.1".to_string(),
                marks: vec![],
            }]);

        let third_node =
            BlockNode::default_list_item(1, ListItemType::Numbered).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 2".to_string(),
                marks: vec![],
            }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
        assert_eq!(&second_node, portabletext_output.get(1).unwrap());
        assert_eq!(&third_node, portabletext_output.get(2).unwrap());
    }

    #[test]
    fn links() {
        let markdown_input = "This is a *[a link](https://github.com)* and more text";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let mark_def_one = match portabletext_output
            .get(0)
            .unwrap()
            .mark_defs
            .get(0)
            .unwrap()
        {
            MarkDef::Link(a) => a,
            _ => panic!(),
        };

        let children = vec![
            SpanNode {
                _type: "span".to_string(),
                text: "This is a ".to_string(),
                marks: vec![],
            },
            SpanNode {
                _type: "span".to_string(),
                text: "a link".to_string(),
                marks: vec![
                    Decorators::Emphasis,
                    Decorators::LinkReference(mark_def_one._key.to_owned()),
                ],
            },
            SpanNode {
                _type: "span".to_string(),
                text: " and more text".to_string(),
                marks: vec![],
            },
        ];

        assert_eq!("https://github.com", mark_def_one.href);
        assert_eq!("link", mark_def_one._type);
        assert_eq!(children, portabletext_output.get(0).unwrap().children);
    }

    #[test]
    fn two_links() {
        let markdown_input = "First we have some text [and some more links](https://www.rust-lang.org/). After which another sentence is started on [abc](//github.com) to finish.";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let mark_def_one = match portabletext_output
            .get(0)
            .unwrap()
            .mark_defs
            .get(0)
            .unwrap()
        {
            MarkDef::Link(a) => a,
            _ => panic!(),
        };

        let mark_def_two = match portabletext_output
            .get(0)
            .unwrap()
            .mark_defs
            .get(1)
            .unwrap()
        {
            MarkDef::Link(a) => a,
            _ => panic!(),
        };

        let children = vec![
            SpanNode {
                _type: "span".to_owned(),
                text: "First we have some text ".to_owned(),
                marks: vec![],
            },
            SpanNode {
                _type: "span".to_owned(),
                text: "and some more links".to_owned(),
                marks: vec![Decorators::LinkReference(mark_def_one._key.to_owned())],
            },
            SpanNode {
                _type: "span".to_owned(),
                text: ". After which another sentence is started on ".to_owned(),
                marks: vec![],
            },
            SpanNode {
                _type: "span".to_owned(),
                text: "abc".to_owned(),
                marks: vec![Decorators::LinkReference(mark_def_two._key.to_owned())],
            },
            SpanNode {
                _type: "span".to_owned(),
                text: " to finish.".to_owned(),
                marks: vec![],
            },
        ];

        assert_eq!("https://www.rust-lang.org/", mark_def_one.href);
        assert_eq!("link", mark_def_one._type);
        assert_eq!(children, portabletext_output.get(0).unwrap().children);
    }

    #[test]
    fn images() {
        let markdown_input = "![The San Juan Mountains are beautiful!](/assets/images/san-juan-mountains.jpg \"San Juan Mountains\")";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let block = portabletext_output.get(0).unwrap();

        if let MarkDef::Image(asset) = block.mark_defs.get(0).as_ref().unwrap() {
            assert_eq!("block", block._type);
            assert_eq!(
                &Decorators::AssetReference(asset._key.to_owned()),
                block.children.get(0).unwrap().marks.get(0).unwrap()
            );
            assert_eq!("/assets/images/san-juan-mountains.jpg", asset.src);
        } else {
            panic!()
        };
    }

    #[test]
    fn linking_images() {
        let markdown_input = "[![An old rock in the desert](/assets/images/shiprock.jpg \"Shiprock, New Mexico by Beau Rogers\")](https://www.flickr.com/photos/beaurogers/31833779864/in/photolist-Qv3rFw-34mt9F-a9Cmfy-5Ha3Zi-9msKdv-o3hgjr-hWpUte-4WMsJ1-KUQ8N-deshUb-vssBD-6CQci6-8AFCiD-zsJWT-nNfsgB-dPDwZJ-bn9JGn-5HtSXY-6CUhAL-a4UTXB-ugPum-KUPSo-fBLNm-6CUmpy-4WMsc9-8a7D3T-83KJev-6CQ2bK-nNusHJ-a78rQH-nw3NvT-7aq2qf-8wwBso-3nNceh-ugSKP-4mh4kh-bbeeqH-a7biME-q3PtTf-brFpgb-cg38zw-bXMZc-nJPELD-f58Lmo-bXMYG-bz8AAi-bxNtNT-bXMYi-bXMY6-bXMYv)";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let block = portabletext_output.get(0).unwrap();

        if let MarkDef::Link(link) = block.mark_defs.get(0).as_ref().unwrap() {
            assert_eq!("block", block._type);
            assert_eq!(
                &Decorators::LinkReference(link._key.to_owned()),
                block.children.get(0).unwrap().marks.get(0).unwrap()
            );
            assert_eq!("https://www.flickr.com/photos/beaurogers/31833779864/in/photolist-Qv3rFw-34mt9F-a9Cmfy-5Ha3Zi-9msKdv-o3hgjr-hWpUte-4WMsJ1-KUQ8N-deshUb-vssBD-6CQci6-8AFCiD-zsJWT-nNfsgB-dPDwZJ-bn9JGn-5HtSXY-6CUhAL-a4UTXB-ugPum-KUPSo-fBLNm-6CUmpy-4WMsc9-8a7D3T-83KJev-6CQ2bK-nNusHJ-a78rQH-nw3NvT-7aq2qf-8wwBso-3nNceh-ugSKP-4mh4kh-bbeeqH-a7biME-q3PtTf-brFpgb-cg38zw-bXMZc-nJPELD-f58Lmo-bXMYG-bz8AAi-bxNtNT-bXMYi-bXMY6-bXMYv", link.href);
        } else {
            panic!()
        };

        if let MarkDef::Image(asset) = block.mark_defs.get(1).as_ref().unwrap() {
            assert_eq!("block", block._type);
            assert_eq!(
                &Decorators::AssetReference(asset._key.to_owned()),
                block.children.get(0).unwrap().marks.get(1).unwrap()
            );
            assert_eq!("/assets/images/shiprock.jpg", asset.src);
        } else {
            panic!()
        };
    }

    #[test]
    fn running_images() {
        let markdown_input = "A running text that then links: [![An old rock in the desert](/assets/images/shiprock.jpg \"Shiprock, New Mexico by Beau Rogers\")](https://www.flickr.com/photos/beaurogers/31833779864/in/photolist-Qv3rFw-34mt9F-a9Cmfy-5Ha3Zi-9msKdv-o3hgjr-hWpUte-4WMsJ1-KUQ8N-deshUb-vssBD-6CQci6-8AFCiD-zsJWT-nNfsgB-dPDwZJ-bn9JGn-5HtSXY-6CUhAL-a4UTXB-ugPum-KUPSo-fBLNm-6CUmpy-4WMsc9-8a7D3T-83KJev-6CQ2bK-nNusHJ-a78rQH-nw3NvT-7aq2qf-8wwBso-3nNceh-ugSKP-4mh4kh-bbeeqH-a7biME-q3PtTf-brFpgb-cg38zw-bXMZc-nJPELD-f58Lmo-bXMYG-bz8AAi-bxNtNT-bXMYi-bXMY6-bXMYv) and continues here";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        assert_eq!(1, portabletext_output.len());

        let image_block = portabletext_output.get(0).unwrap();

        if let MarkDef::Image(asset) = image_block.mark_defs.get(1).as_ref().unwrap() {
            assert_eq!("block", image_block._type);
            assert_eq!(
                &Decorators::AssetReference(asset._key.to_owned()),
                image_block.children.get(1).unwrap().marks.get(1).unwrap()
            );
            assert_eq!("/assets/images/shiprock.jpg", asset.src);
        } else {
            panic!()
        };
    }

    #[test]
    fn inline_code() {
        let markdown_input = "A piece of inline `code snippet` marked by a backtick";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let block = portabletext_output.get(0).unwrap();

        assert_eq!(
            block.children.get(1).unwrap().marks.get(0).unwrap(),
            &Decorators::Code
        );
    }

    #[test]
    fn fenced_code_with_lang() {
        let markdown_input = "```rust\nprintln!(\"{:?}\", my_var)\n```";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let block = portabletext_output.get(0).unwrap();

        assert_eq!(block.language.as_ref().unwrap(), "rust");
        assert_eq!(block.code.as_ref().unwrap(), "println!(\"{:?}\", my_var)\n")
    }

    #[test]
    #[cfg(feature = "serde_serialization")]
    fn serialization() {
        let markdown_input = "A running text that then links";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let j = serde_json::to_string(&portabletext_output).unwrap();

        assert_eq!(j, "[{\"_type\":\"block\",\"style\":\"normal\",\"children\":[{\"_type\":\"span\",\"text\":\"A running text that then links\",\"marks\":[]}],\"markDefs\":[]}]");
    }

    #[test]
    #[cfg(feature = "serde_serialization")]
    fn lowercased_enums() {
        assert_eq!(
            "\"em\"",
            serde_json::to_string(&Decorators::Emphasis).unwrap()
        );
        assert_eq!(
            "\"numbered\"",
            serde_json::to_string(&ListItemType::Numbered).unwrap()
        );

        assert_eq!(
            "\"a23xas\"",
            serde_json::to_string(&Decorators::LinkReference("a23xas".to_owned())).unwrap()
        );
    }
}
