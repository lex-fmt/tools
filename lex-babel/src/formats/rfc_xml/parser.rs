use crate::error::FormatError;
use crate::ir::nodes::{
    DocNode, Document, Heading, InlineContent, List, ListItem, Paragraph, Verbatim, Definition,
};
use roxmltree::{Node, NodeType};

pub fn parse_to_ir(source: &str) -> Result<Document, FormatError> {
    let doc = roxmltree::Document::parse(source).map_err(|e| {
        FormatError::ParseError(format!("XML parsing error: {}", e))
    })?;

    let root = doc.root_element();
    // RFC XML root is <rfc> or sometimes <internet-draft> (older?) -> spec says <rfc>
    if root.tag_name().name() != "rfc" {
        return Err(FormatError::ParseError(
            format!("Root element is <{}>, expected <rfc>", root.tag_name().name())
        ));
    }

    let mut children = Vec::new();

    // Process <front>
    if let Some(front) = root.children().find(|n| n.tag_name().name() == "front") {
        children.extend(process_front(front)?);
    }

    // Process <middle>
    if let Some(middle) = root.children().find(|n| n.tag_name().name() == "middle") {
        children.extend(process_middle(middle)?);
    }

    // Process <back>
    if let Some(back) = root.children().find(|n| n.tag_name().name() == "back") {
        children.extend(process_back(back)?);
    }

    Ok(Document { children })
}

fn process_front(node: Node) -> Result<Vec<DocNode>, FormatError> {
    let mut nodes = Vec::new();

    // Title
    if let Some(title) = node.children().find(|n| n.tag_name().name() == "title") {
        nodes.push(DocNode::Heading(Heading {
            level: 1,
            content: parse_inline_content(title)?,
            children: Vec::new(),
        }));
    }

    // Abstract
    if let Some(abstract_node) = node.children().find(|n| n.tag_name().name() == "abstract") {
        nodes.push(DocNode::Heading(Heading {
            level: 2,
            content: vec![InlineContent::Text("Abstract".to_string())],
            children: process_container_children(abstract_node, 2)?,
        }));
    }

    Ok(nodes)
}

fn process_middle(node: Node) -> Result<Vec<DocNode>, FormatError> {
    process_container_children(node, 1)
}

fn process_back(node: Node) -> Result<Vec<DocNode>, FormatError> {
     // <back> usually contains <references> and <section> (Appendices)
     // We treat them as Level 1 sections (Appendices)
     process_container_children(node, 1)
}

fn process_container_children(node: Node, current_level: usize) -> Result<Vec<DocNode>, FormatError> {
    let mut nodes = Vec::new();
    for child in node.children() {
        if child.node_type() != NodeType::Element {
            continue;
        }

        match child.tag_name().name() {
            "section" => {
                let title_text = child.attribute("title")
                    .map(|s| vec![InlineContent::Text(s.to_string())])
                    .or_else(|| {
                         child.children().find(|n| n.tag_name().name() == "name")
                             .map(|n| parse_inline_content(n).unwrap_or_default())
                    })
                    .unwrap_or_else(|| vec![InlineContent::Text("Untitled".to_string())]);

                nodes.push(DocNode::Heading(Heading {
                    level: current_level + 1,
                    content: title_text,
                    children: process_container_children(child, current_level + 1)?,
                }));
            }
            "t" => {
                nodes.push(DocNode::Paragraph(Paragraph {
                    content: parse_inline_content(child)?,
                }));
            }
            "ul" | "ol" => {
                 let ordered = child.tag_name().name() == "ol";
                 nodes.push(DocNode::List(List {
                     ordered,
                     items: process_list_items(child, current_level)?,
                 }));
            }
            "list" => {
                let style = child.attribute("style").unwrap_or("empty");
                let ordered = style == "numbers" || style == "letters" || style == "format";
                nodes.push(DocNode::List(List {
                    ordered,
                    items: process_list_items(child, current_level)?,
                }));
            }
             "dl" => {
                 nodes.extend(process_definition_list(child, current_level)?);
             }
             "figure" => {
                  // Figure can contain optional <name> (caption), <artwork>/<sourcecode>, <postamble>
                  // We extract artwork/sourcecode.
                  // TODO: Handle caption
                  if let Some(artwork) = child.children().find(|n| n.tag_name().name() == "artwork" || n.tag_name().name() == "sourcecode") {
                      nodes.push(parse_verbatim(artwork)?);
                  }
             }
             "artwork" | "sourcecode" => {
                 nodes.push(parse_verbatim(child)?);
             }
             "note" => {
                  let title_text = child.attribute("title")
                     .map(|s| vec![InlineContent::Text(s.to_string())])
                     .or_else(|| {
                          child.children().find(|n| n.tag_name().name() == "name")
                              .map(|n| parse_inline_content(n).unwrap_or_default())
                     })
                     .unwrap_or_else(|| vec![InlineContent::Text("Note".to_string())]);
 
                 nodes.push(DocNode::Heading(Heading {
                     level: current_level + 1,
                     content: title_text,
                     children: process_container_children(child, current_level + 1)?,
                 }));
             }
             "references" => {
                 // References section
                 let title_text = child.attribute("title")
                     .map(|s| vec![InlineContent::Text(s.to_string())])
                     .or_else(|| {
                          child.children().find(|n| n.tag_name().name() == "name")
                              .map(|n| parse_inline_content(n).unwrap_or_default())
                     })
                     .unwrap_or_else(|| vec![InlineContent::Text("References".to_string())]);

                 nodes.push(DocNode::Heading(Heading {
                     level: current_level + 1,
                     content: title_text,
                     children: process_container_children(child, current_level + 1)?,
                 }));
             }
             "reference" => {
                  // A single reference entry. In RFC XML, these are complex.
                  // For now, render as a Paragraph with the reference info.
                  // TODO: Better citation formatting.
                  let anchor = child.attribute("anchor").unwrap_or("?");
                  let mut content = vec![InlineContent::Reference(format!("[{}]", anchor)), InlineContent::Text(" ".to_string())];
                  
                  // Try to find title
                  if let Some(front) = child.children().find(|n| n.tag_name().name() == "front") {
                      if let Some(title) = front.children().find(|n| n.tag_name().name() == "title") {
                          content.extend(parse_inline_content(title)?);
                      }
                  }
                  
                  nodes.push(DocNode::Paragraph(Paragraph { content }));
             }
            _ => {
                // Skip unknown
            }
        }
    }
    Ok(nodes)
}

fn process_list_items(node: Node, level: usize) -> Result<Vec<ListItem>, FormatError> {
    let mut items = Vec::new();
    for child in node.children() {
        if child.node_type() != NodeType::Element { continue; }
        
        if child.tag_name().name() == "li" {
             // <li> content is children nodes (usually <t>)
             let mut children = process_container_children(child, level)?;
             let mut content = Vec::new();

             // Optimization: If the list item contains exactly one paragraph,
             // hoist its content to the list item's inline content.
             // This produces "- Item" instead of "-\n    Item".
             if children.len() == 1 {
                 if let Some(DocNode::Paragraph(para)) = children.first() {
                     content = para.content.clone();
                     children.clear();
                 }
             }

             items.push(ListItem {
                 content,
                 children,
             });
        } else if child.tag_name().name() == "t" {
              // v2 style: <list><t>Item</t></list>
              // Here <t> is the item.
              items.push(ListItem {
                 content: parse_inline_content(child)?,
                 children: Vec::new(),
             });
        }
    }
    Ok(items)
}

fn process_definition_list(node: Node, level: usize) -> Result<Vec<DocNode>, FormatError> {
    // <dl> contains sequence of <dt>, <dd>.
    let mut definitions = Vec::new();
    let mut current_term = Vec::new();
    
    for child in node.children() {
         if child.node_type() != NodeType::Element { continue; }
         
         if child.tag_name().name() == "dt" {
             current_term = parse_inline_content(child)?;
         } else if child.tag_name().name() == "dd" {
             // Description
             let description = process_container_children(child, level)?;
             definitions.push(DocNode::Definition(Definition {
                 term: current_term.clone(),
                 description,
             }));
             current_term.clear(); // Reset (though XML validly has dt, dd, dt, dd...)
         }
    }
    Ok(definitions)
}

fn parse_verbatim(node: Node) -> Result<DocNode, FormatError> {
    // text() gets direct text. For CDATA or mixed content it works differently in roxmltree?
    // roxmltree text() returns content if it's text node. 
    // We need to collect text from all children if it's mixed?
    // Usually sourcecode is pure text.
    let text = node.text().unwrap_or("").to_string();
    Ok(DocNode::Verbatim(Verbatim {
        language: node.attribute("type").map(|s| s.to_string()),
        content: text,
    }))
}

fn parse_inline_content(node: Node) -> Result<Vec<InlineContent>, FormatError> {
    let mut content = Vec::new();
    for child in node.children() {
        match child.node_type() {
            NodeType::Text => {
                let text = child.text().unwrap_or("");
                // Normalize whitespace? RFC XML usually has significant whitespace only in verbatim.
                // In <t>, newlines might be just wrapping.
                // roxmltree preserves whitespace.
                if !text.is_empty() {
                    content.push(InlineContent::Text(text.to_string()));
                }
            }
            NodeType::Element => {
                match child.tag_name().name() {
                    "strong" | "b" => {
                        content.push(InlineContent::Bold(parse_inline_content(child)?));
                    }
                    "em" | "i" => {
                        content.push(InlineContent::Italic(parse_inline_content(child)?));
                    }
                    "tt" | "code" => {
                         // code in v3 can be inline
                         content.push(InlineContent::Code(child.text().unwrap_or("").to_string()));
                    }
                    "xref" => {
                        let target = child.attribute("target").unwrap_or("?");
                        // If has text content, use it, else use target
                        let mut inner_text = String::new();
                         for c in child.children() {
                             if c.node_type() == NodeType::Text {
                                 inner_text.push_str(c.text().unwrap_or(""));
                             }
                         }
                        
                        let text_to_show = if !inner_text.trim().is_empty() {
                            inner_text
                        } else {
                            target.to_string()
                        };
                        
                        content.push(InlineContent::Reference(text_to_show)); 
                    }
                    "eref" => {
                         let target = child.attribute("target").unwrap_or("?");
                         content.push(InlineContent::Reference(target.to_string()));
                    }
                    _ => {
                        // Recurse transparently
                         content.extend(parse_inline_content(child)?);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(content)
}