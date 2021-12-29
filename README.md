# pulldown-cmark-portabletext

A first attempt to make structure out of markdown and make it more portable. Especially when front matter is used to enrich the model - going direct ot html makes a lot of assumptions. Let's see how far this approach brings me.

## Example

```
use pulldown_cmark_portabletext::portabletext;
use pulldown_cmark_portabletext::portabletext::{BlockNode, Decorators, ListItemType, SpanNode};
use pulldown_cmark::{Options, Parser};

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
```

## References
* https://astexplorer.net/
* https://codesandbox.io/s/ancient-cherry-yjqou
* also cool: https://github.com/syntax-tree/mdast