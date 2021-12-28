// TODOS:
// remove .unwraps() to avoid panics
// remove the _ catch of pattern match

pub mod portabletext {
    use std::io;

    use pulldown_cmark::Event::*;
    use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    #[derive(Debug, PartialEq, Clone)]
    pub struct MarkDef {
        pub _key: String,
        pub _type: String,
        pub href: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Asset {
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
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum ListItemType {
        Bullit,
        Numbered,
    }

    #[derive(Debug, PartialEq)]
    pub struct SpanNode {
        pub _type: String,
        pub text: String,
        pub marks: Vec<Decorators>,
    }
    #[derive(Debug, PartialEq)]
    pub struct BlockNode {
        pub _type: String,
        pub style: String,
        // strictly not required on ever node, let check if we can optimze this later
        // tho in rust Vec::with_capacity(0) is zero bytes
        pub children: Vec<SpanNode>,

        // meta on marks
        pub mark_defs: Vec<MarkDef>,
        // list items
        pub level: Option<usize>,
        pub list_item: Option<ListItemType>,

        pub asset: Option<Asset>,
    }

    // TODO: split this into multiple types
    impl BlockNode {
        pub fn default(style: String) -> Self {
            Self {
                _type: "block".to_string(),
                style: style,
                children: Vec::with_capacity(0),
                mark_defs: Vec::with_capacity(0),
                asset: None,
                level: None,
                list_item: None,
            }
        }

        pub fn default_list_item(level: usize, list_item: ListItemType) -> Self {
            Self {
                _type: "block".to_string(),
                style: "normal".to_string(),
                level: Some(level),
                list_item: Some(list_item),
                children: Vec::with_capacity(2),
                mark_defs: Vec::with_capacity(0),
                asset: None,
            }
        }

        pub fn with_children(mut self, children: Vec<SpanNode>) -> Self {
            self.children = children;
            self
        }
    }

    struct PortabletextWriter<'a, I> {
        /// Iterator supplying events.
        iter: I,
        /// Writer to write to.
        writer: &'a mut Vec<BlockNode>,
        /// Whether or not the last write wrote a newline.
        open_block: bool,
        active_list_item: Vec<ListItemType>,
        list_item_level: usize,
        active_markers: Vec<Decorators>, // table_state: TableState,
                                         // table_alignments: Vec<Alignment>,
                                         // table_cell_index: usize,
                                         // numbers: HashMap<CowStr<'a>, usize>
    }
    impl<'a, I> PortabletextWriter<'a, I>
    where
        I: Iterator<Item = Event<'a>>,
        // W: ObjWrite,
    {
        fn new(iter: I, writer: &'a mut Vec<BlockNode>) -> Self {
            Self {
                iter,
                writer,
                open_block: false,
                active_markers: Vec::with_capacity(3),
                // table_state: TableState::Head,
                // table_alignments: vec![],
                // table_cell_index: 0,
                // numbers: HashMap::new(),
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
                println!("{:?}", event);
                match event {
                    Start(tag) => {
                        self.start_tag(tag)?;
                    }
                    End(tag) => {
                        self.end_tag(tag)?;
                    }
                    Text(text) => {
                        self.add_span(text)?;
                    }
                    Code(_) | Html(_) | FootnoteReference(_) | Rule | SoftBreak | HardBreak
                    | TaskListMarker(_) => {}
                }
            }
            Ok(())
        }

        fn consume_inner(&mut self) -> String {
            let mut nest = 0;
            let mut buffer = String::new();
            while let Some(event) = self.iter.next() {
                match event {
                    Start(_) => nest += 1,
                    End(_) => {
                        if nest == 0 {
                            break;
                        }
                        nest -= 1;
                    }
                    Html(text) | Code(text) | Text(text) => {
                        buffer.push_str(&text.to_string());
                    }
                    SoftBreak | HardBreak | Rule => {
                        buffer.push_str(" ");
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
                Tag::CodeBlock(CodeBlockKind::Fenced(_syntax)) => {
                    // todo: add mark def
                    self.write(BlockNode::default("code".to_string()))
                }
                Tag::CodeBlock(CodeBlockKind::Indented) => {
                    self.write(BlockNode::default("code".to_string()))
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
                    let mark_def = MarkDef {
                        _type: "link".to_owned(),
                        _key: key.to_owned(),
                        href: link_href.to_string(),
                    };
                    self.add_mark_def(mark_def).unwrap();
                    self.mark_start(Decorators::LinkReference(key))
                }
                Tag::Image(_image_type, image_href, title) => {
                    let key: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(12)
                        .map(char::from)
                        .collect();

                    let asset = Asset {
                        _ref: key,
                        src: image_href.to_string(),
                    };

                    let alt = self.consume_inner();
                    let last_block = self.last_block();
                    last_block._type = "image".to_owned();
                    last_block.asset = Some(asset);

                    if !title.is_empty() {
                        last_block.children.push(SpanNode {
                            _type: "image-title".to_owned(),
                            marks: Vec::with_capacity(0),
                            text: title.to_string(),
                        });
                    }

                    last_block.children.push(SpanNode {
                        _type: "image-alt".to_owned(),
                        marks: Vec::with_capacity(0),
                        text: alt.to_string(),
                    });

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
                Tag::Strong => self.mark_stop(),
                Tag::Emphasis => self.mark_stop(),
                Tag::Strikethrough => self.mark_stop(),
                Tag::List(_options) => {
                    self.active_list_item.pop();
                    self.list_item_level -= 1;
                    Ok(())
                }
                _ => self.close_block(),
            }
        }

        fn add_span(&mut self, text: CowStr<'a>) -> io::Result<()> {
            self.add_span_with_type(text, "span".to_owned())
        }

        fn add_span_with_type(&mut self, text: CowStr<'a>, _type: String) -> io::Result<()> {
            let marks: Vec<Decorators> = self.active_markers.to_vec();
            let current_node = self.last_block();
            current_node.children.push(SpanNode {
                _type: _type,
                text: text.to_string(),
                marks: marks,
            });

            Ok(())
        }

        fn close_block(&mut self) -> io::Result<()> {
            self.open_block = false;
            Ok(())
        }

        fn last_block(&mut self) -> &mut BlockNode {
            let length = self.writer.len();
            self.writer.get_mut(length - 1).unwrap()
        }

        fn add_mark_def(&mut self, mark_def: MarkDef) -> io::Result<()> {
            self.last_block().mark_defs.push(mark_def);
            Ok(())
        }

        fn mark_start(&mut self, decorator: Decorators) -> io::Result<()> {
            self.active_markers.push(decorator);
            Ok(())
        }

        fn mark_stop(&mut self) -> io::Result<()> {
            // assumes balanced tags
            self.active_markers.pop();
            Ok(())
        }
    }

    pub fn push_portabletext<'a, I>(output: &'a mut Vec<BlockNode>, parser: I)
    where
        I: Iterator<Item = Event<'a>>,
    {
        PortabletextWriter::new(parser, output).run().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::portabletext;
    use crate::portabletext::{BlockNode, Decorators, ListItemType, SpanNode};
    use pulldown_cmark::{Options, Parser};

    #[test]
    fn it_works() {
        let markdown_input = "Hello world, this is a ~~complicated~~ *very simple* example.";

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(markdown_input, options);

        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("normal".to_string()).with_children(vec![
            SpanNode {
                _type: "span".to_string(),
                text: "Hello world, this is a ".to_string(),
                marks: Vec::with_capacity(0),
            },
            SpanNode {
                _type: "span".to_string(),
                text: "complicated".to_string(),
                marks: vec![Decorators::Strike],
            },
            SpanNode {
                _type: "span".to_string(),
                text: " ".to_string(),
                marks: Vec::with_capacity(0),
            },
            SpanNode {
                _type: "span".to_string(),
                text: "very simple".to_string(),
                marks: vec![Decorators::Emphasis],
            },
            SpanNode {
                _type: "span".to_string(),
                text: " example.".to_string(),
                marks: Vec::with_capacity(0),
            },
        ]);
        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
    }

    #[test]
    fn it_supports_heading() {
        let markdown_input = "# Hey";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let first_node = BlockNode::default("h1".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "Hey".to_string(),
            marks: Vec::with_capacity(0),
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
            marks: Vec::with_capacity(0),
        }]);

        let second_node = BlockNode::default("h2".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "HeyHey".to_string(),
            marks: Vec::with_capacity(0),
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
            marks: Vec::with_capacity(0),
        }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
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
                marks: Vec::with_capacity(0),
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
                marks: Vec::with_capacity(0),
            }]);

        let second_node = BlockNode::default("normal".to_string()).with_children(vec![SpanNode {
            _type: "span".to_string(),
            text: "Hi there".to_string(),
            marks: Vec::with_capacity(0),
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

        let first_node = BlockNode::default("blockquote".to_string()).with_children(vec![
            SpanNode {
                _type: "span".to_string(),
                text: "Okay, pep talk!".to_string(),
                marks: Vec::with_capacity(0),
            },
            SpanNode {
                _type: "span".to_string(),
                text: "Hi there".to_string(),
                marks: Vec::with_capacity(0),
            },
        ]);

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
                marks: Vec::with_capacity(0),
            }]);

        let second_node =
            BlockNode::default_list_item(2, ListItemType::Bullit).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 1.1".to_string(),
                marks: Vec::with_capacity(0),
            }]);

        let third_node =
            BlockNode::default_list_item(1, ListItemType::Bullit).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 2".to_string(),
                marks: Vec::with_capacity(0),
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
                marks: Vec::with_capacity(0),
            }]);

        let second_node =
            BlockNode::default_list_item(2, ListItemType::Numbered).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 1.1".to_string(),
                marks: Vec::with_capacity(0),
            }]);

        let third_node =
            BlockNode::default_list_item(1, ListItemType::Numbered).with_children(vec![SpanNode {
                _type: "span".to_string(),
                text: "Item 2".to_string(),
                marks: Vec::with_capacity(0),
            }]);

        assert_eq!(&first_node, portabletext_output.get(0).unwrap());
        assert_eq!(&second_node, portabletext_output.get(1).unwrap());
        assert_eq!(&third_node, portabletext_output.get(2).unwrap());
    }

    #[test]
    fn links() {
        let markdown_input = "This is a *[a link](https://github.com)*";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let mark_def = portabletext_output
            .get(0)
            .unwrap()
            .mark_defs
            .get(0)
            .unwrap();
        let children = vec![
            SpanNode {
                _type: "span".to_string(),
                text: "This is a ".to_string(),
                marks: Vec::with_capacity(0),
            },
            SpanNode {
                _type: "span".to_string(),
                text: "a link".to_string(),
                marks: vec![
                    Decorators::Emphasis,
                    Decorators::LinkReference(mark_def._key.to_owned()),
                ],
            },
        ];

        assert_eq!("https://github.com", mark_def.href);
        assert_eq!("link", mark_def._type);
        assert_eq!(children, portabletext_output.get(0).unwrap().children);
    }

    #[test]
    fn images() {
        let markdown_input = "![The San Juan Mountains are beautiful!](/assets/images/san-juan-mountains.jpg \"San Juan Mountains\")";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let block = portabletext_output.get(0).unwrap();
        let asset = block.asset.as_ref().unwrap();

        assert_eq!("image", block._type);
        assert_eq!("/assets/images/san-juan-mountains.jpg", asset.src);
    }

    #[test]
    fn linking_images() {
        let markdown_input = "[![An old rock in the desert](/assets/images/shiprock.jpg \"Shiprock, New Mexico by Beau Rogers\")](https://www.flickr.com/photos/beaurogers/31833779864/in/photolist-Qv3rFw-34mt9F-a9Cmfy-5Ha3Zi-9msKdv-o3hgjr-hWpUte-4WMsJ1-KUQ8N-deshUb-vssBD-6CQci6-8AFCiD-zsJWT-nNfsgB-dPDwZJ-bn9JGn-5HtSXY-6CUhAL-a4UTXB-ugPum-KUPSo-fBLNm-6CUmpy-4WMsc9-8a7D3T-83KJev-6CQ2bK-nNusHJ-a78rQH-nw3NvT-7aq2qf-8wwBso-3nNceh-ugSKP-4mh4kh-bbeeqH-a7biME-q3PtTf-brFpgb-cg38zw-bXMZc-nJPELD-f58Lmo-bXMYG-bz8AAi-bxNtNT-bXMYi-bXMY6-bXMYv)";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        let block = portabletext_output.get(0).unwrap();
        let asset = block.asset.as_ref().unwrap();

        assert_eq!("image", block._type);
        assert_eq!("/assets/images/shiprock.jpg", asset.src);
    }

    #[test]
    fn running_images() {
        let markdown_input = "A running text that then links: [![An old rock in the desert](/assets/images/shiprock.jpg \"Shiprock, New Mexico by Beau Rogers\")](https://www.flickr.com/photos/beaurogers/31833779864/in/photolist-Qv3rFw-34mt9F-a9Cmfy-5Ha3Zi-9msKdv-o3hgjr-hWpUte-4WMsJ1-KUQ8N-deshUb-vssBD-6CQci6-8AFCiD-zsJWT-nNfsgB-dPDwZJ-bn9JGn-5HtSXY-6CUhAL-a4UTXB-ugPum-KUPSo-fBLNm-6CUmpy-4WMsc9-8a7D3T-83KJev-6CQ2bK-nNusHJ-a78rQH-nw3NvT-7aq2qf-8wwBso-3nNceh-ugSKP-4mh4kh-bbeeqH-a7biME-q3PtTf-brFpgb-cg38zw-bXMZc-nJPELD-f58Lmo-bXMYG-bz8AAi-bxNtNT-bXMYi-bXMY6-bXMYv) and continues here";

        let parser = Parser::new(markdown_input);
        let mut portabletext_output = vec![];
        portabletext::push_portabletext(&mut portabletext_output, parser);

        assert_eq!(1, portabletext_output.len());

        let image_block = portabletext_output.get(0).unwrap();
        let asset = image_block.asset.as_ref().unwrap();

        assert_eq!("image", image_block._type);
        assert_eq!("/assets/images/shiprock.jpg", asset.src);
    }
}
