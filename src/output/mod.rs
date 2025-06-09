pub mod markdown;
pub mod codeblock;

#[cfg(test)]
#[path = "markdown.test.rs"]
mod markdown_test;

#[cfg(test)]
#[path = "markdown_formatting_demo.test.rs"]
mod markdown_formatting_demo_test;

pub use markdown::{MarkdownGenerator, MarkdownTemplate, MarkdownConfig};
pub use codeblock::{CodeBlockGenerator, CodeBlockConfig, CodeBlock, CodeBlockType};