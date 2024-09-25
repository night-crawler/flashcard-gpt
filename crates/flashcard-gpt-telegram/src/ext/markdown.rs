use anyhow::anyhow;
use itertools::Itertools;
use markdown::mdast::Node;
use markdown::ParseOptions;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct MarkdownFormatter {
    opts: Arc<Mutex<ParseOptions>>,
}

unsafe impl Send for MarkdownFormatter {}
unsafe impl Sync for MarkdownFormatter {}

impl MarkdownFormatter {
    pub fn new(opts: ParseOptions) -> Self {
        Self {
            opts: Arc::new(Mutex::new(opts)),
        }
    }

    pub fn to_html(&self, text: &str) -> anyhow::Result<String> {
        let opts = self.opts.lock().map_err(|e| anyhow!("{e:?}"))?;
        let ast = markdown::to_mdast(text, &opts).map_err(|e| anyhow!("{e:?}"))?;
        Ok(ast.to_tg_html())
    }
}

pub trait TgHtml {
    fn to_tg_html(&self) -> String;
}

impl TgHtml for Node {
    fn to_tg_html(&self) -> String {
        match self {
            Node::Root(root) => root
                .children
                .iter()
                .map(|child| child.to_tg_html())
                .join("\n"),
            Node::Paragraph(paragraph) => paragraph
                .children
                .iter()
                .map(|child| child.to_tg_html())
                .collect(),
            Node::Text(text) => escape_html(&text.value),
            Node::Emphasis(emphasis) => {
                let content = emphasis
                    .children
                    .iter()
                    .map(|child| child.to_tg_html())
                    .collect::<String>();
                format!("<i>{content}</i>")
            }
            Node::Strong(strong) => {
                let content = strong
                    .children
                    .iter()
                    .map(|child| child.to_tg_html())
                    .collect::<String>();
                format!("<b>{content}</b>")
            }
            Node::Delete(delete) => {
                let content = delete
                    .children
                    .iter()
                    .map(|child| child.to_tg_html())
                    .collect::<String>();
                format!("<s>{content}</s>")
            }
            Node::InlineCode(inline_code) => {
                let content = escape_html(&inline_code.value);
                format!("<code>{}</code>", content)
            }
            Node::Code(code) => {
                let content = escape_html(&code.value);
                if let Some(lang) = &code.lang {
                    format!(
                        "<pre><code class=\"language-{}\">{content}</code></pre>",
                        escape_html(lang),
                    )
                } else {
                    format!("<pre>{content}</pre>")
                }
            }
            Node::Link(link) => {
                let href = escape_html(&link.url);
                let content = link
                    .children
                    .iter()
                    .map(|child| child.to_tg_html())
                    .collect::<String>();
                format!(r#"<a href="{href}">{content}</a>"#)
            }
            Node::LinkReference(link_ref) => {
                let content = link_ref
                    .children
                    .iter()
                    .map(|child| child.to_tg_html())
                    .collect::<String>();
                content
            }
            Node::Image(image) => {
                let alt = escape_html(&image.alt);
                let url = &image.url;
                format!(r#"<a href="{url}">{alt}</a>"#)
            }
            Node::ImageReference(image_ref) => {
                let alt = escape_html(&image_ref.alt);
                format!("[{}]", alt)
            }
            Node::BlockQuote(block_quote) => {
                let content = block_quote
                    .children
                    .iter()
                    .map(|child| child.to_tg_html())
                    .join("\n");
                format!("<blockquote>{content}</blockquote>")
            }
            Node::Heading(heading) => {
                let content = heading
                    .children
                    .iter()
                    .map(|child| child.to_tg_html())
                    .collect::<String>();
                format!("<b>{content}</b>")
            }
            Node::List(list) => {
                let mut items = vec![];
                for (index, item) in list.children.iter().enumerate() {
                    let item_content = item.to_tg_html();
                    if list.ordered {
                        items.push(format!("{}. {}", index + 1, item_content));
                    } else {
                        items.push(format!(" -{}", item_content));
                    }
                }
                items.join("\n")
            }
            Node::ListItem(li) => li
                .children
                .iter()
                .map(|child| format!(" {}", child.to_tg_html()))
                .join("\n"),
            Node::Break(_) => "\n".to_string(),
            Node::Html(html_node) => escape_html(&html_node.value),
            Node::ThematicBreak(_) => "\n---\n".to_string(),
            Node::InlineMath(inline_math) => {
                let content = escape_html(&inline_math.value);
                format!("<code>{content}</code>")
            }
            Node::Math(math) => {
                let content = escape_html(&math.value);
                format!("<pre>{content}</pre>")
            }
            Node::Table(table) => {
                let mut result = String::new();
                result.push_str("<pre>");
                for row in &table.children {
                    if let Node::TableRow(row) = row {
                        let mut row_content = String::new();
                        for cell in &row.children {
                            if let Node::TableCell(cell) = cell {
                                let cell_content = cell
                                    .children
                                    .iter()
                                    .map(|child| child.to_tg_html())
                                    .collect::<String>();
                                if !row_content.is_empty() {
                                    row_content.push('\t');
                                }
                                row_content.push_str(&cell_content);
                            }
                        }
                        if !result.is_empty() {
                            result.push('\n');
                        }
                        result.push_str(&row_content);
                    }
                }
                result.push_str("\n</pre>");

                result
            }
            Node::FootnoteReference(reference) => {
                let label = escape_html(reference.label.as_ref().unwrap_or(&reference.identifier));
                format!(" [^{label}]")
            }
            Node::FootnoteDefinition(def) => {
                let label = escape_html(def.label.as_ref().unwrap_or(&def.identifier));
                let content = def
                    .children
                    .iter()
                    .map(|child| child.to_tg_html())
                    .collect::<String>();
                format!("[^{label}]: {content}")
            }
            Node::Definition(def) => {
                let label = escape_html(def.label.as_ref().unwrap_or(&def.identifier));
                let title = def
                    .title
                    .as_ref()
                    .map_or(String::new(), |title| escape_html(title));
                let title = [label, title].join(" | ");
                let url = def.url.as_str();
                format!(r#"<a href="{url}">{title}</a>"#)
            }
            Node::MdxJsxFlowElement(_)
            | Node::MdxJsxTextElement(_)
            | Node::MdxTextExpression(_)
            | Node::MdxFlowExpression(_)
            | Node::MdxjsEsm(_)
            | Node::Toml(_)
            | Node::Yaml(_) => String::new(),
            _ => self.children().map_or(String::new(), |children| {
                children.iter().map(|child| child.to_tg_html()).collect()
            }),
        }
    }
}

fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::{Constructs, ParseOptions};
    use testresult::TestResult;

    #[test]
    fn test_bla() -> TestResult {
        let text = r#"
# Sample Document

||spoiler||

---

```yaml
title: Sample Document
author: Assistant
```

```toml
+++
title = "Sample Document"
author = "Assistant"
+++
```

```mdx
import React from 'react';
```

# Heading Level 1

## Heading Level 2

This is a paragraph with **bold** text, *emphasized* text, and ~~strikethrough~~ text.
It also contains `inline code`, an inline math expression $E = mc^2$, and a
[link](https://example.com). Here's a reference to [another link][link_ref].

This line ends with a
line break.

This is an HTML block:

<div>
  This is an HTML block.
</div>

This is an MDX JSX Flow Element:

<CustomComponent prop="value">
  Content inside the custom component.
</CustomComponent>

This is an MDX JSX Text Element: <span style="color:red">Red Text</span>

Here is an image:

![Alt text](https://example.com/image.jpg "Optional title")

Here is an image reference:

![Alt text][image_ref]

Here is a footnote reference[^1].

> This is a blockquote.
>
> It can span multiple lines.

Here is a list:

- Unordered item 1
  - Nested unordered item 1.1
  - Nested unordered item 1.2
    - Nested unordered item 1.2.1
    - Nested unordered item 1.2.2
  - Nested unordered item 1.3
- Unordered item 2
- Unordered item 3
- Unordered item 4

Here is an ordered list:

1. Ordered item 1
2. Ordered item 2

Here is a code block with language specification:

```python
def hello_world():
    print("Hello, world!")
```

Here is a code block without language:

```
This is a code block without language.
```

Here is a math block:

$$
E = mc^2
$$

Here is an MDX flow expression:

{ /* This is an MDX flow expression */ }

Here is a table:

| Syntax    | Description |
|-----------|-------------|
| Header    | Title       |
| Paragraph | Text        |

---

[link_ref]: https://example.com "Optional title"

[image_ref]: https://example.com/image2.jpg "Optional title"

[^1]: This is the footnote definition.
"#;

        let parse_opts = ParseOptions {
            constructs: Constructs {
                math_flow: true,
                math_text: true,
                ..Constructs::gfm()
            },
            ..ParseOptions::gfm()
        };
        let markdown_formatter = MarkdownFormatter::new(parse_opts);
        let html = markdown_formatter.to_html(text)?;
        println!("{}", html);

        Ok(())
    }
}
