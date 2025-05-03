use crate::impls::ToStringRef;
use crate::smali::{tokens_to_raw, SmaliNode, SmaliToken};

impl SmaliNode {
    pub(crate) fn render_internal(&self, ident_level: usize, result: &mut String) {
        let indent_str = "    ".repeat(ident_level);
        result.push_str(&indent_str);
        if let Some(offset_hint) = self.offset_hint {
            result.push_str(&offset_hint.to_string());
            result.push_str(": ");
        }
        let tag = self.tag;
        if let Some(tag) = tag {
            result.push_str(&tag.to_string());
            result.push(' ');
        }
        let content = &self.content;
        if !content.is_empty() {
            result.push_str(&tokens_to_raw(content));
            result.push(' ')
        }

        if self.children.is_empty() && self.end_tag.is_none() {
            return;
        }
        for child in &self.children {
            result.push('\n');
            child.render_internal(ident_level + 1, result);
        }
        if let Some(postfix) = &self.end_tag {
            result.push('\n');
            result.push_str(&indent_str);
            result.push_str(&postfix);
        }
    }

    // render the smali node to multiple lines.
    pub(crate) fn render_to_lines_internal(&self) -> Vec<Vec<SmaliToken>> {
        let max_offset_len = max_offset_hint(self).to_string().len();
        render_to_lines(self, 0, max_offset_len)
    }
}

fn render_to_lines(
    node: &SmaliNode, ident_width: usize, max_offset_len: usize,
) -> Vec<Vec<SmaliToken>> {
    let mut lines = Vec::new();

    let mut current_line = Vec::new();
    let SmaliNode { tag, content, children, end_tag, .. } = node;
    current_line.push(offset_or_stub(max_offset_len, node));
    current_line.push(indent(ident_width));
    if let Some(tag) = tag {
        current_line.push(SmaliToken::Raw(tag));
        current_line.push(SmaliToken::Raw(" "));
    }
    for token in content {
        current_line.push(token.clone());
        current_line.push(SmaliToken::Raw(" "));
    }
    lines.push(current_line);

    for child in children {
        let child_lines = render_to_lines(child, ident_width + 2, max_offset_len);
        lines.extend(child_lines);
    }
    if children.len() > 0 {
        lines.push(vec![])
    }
    if let Some(postfix) = end_tag {
        lines.push(vec![
            indent(ident_width),
            SmaliToken::Raw(postfix),
        ]);
    }
    lines
}

fn max_offset_hint(smali_node: &SmaliNode) -> u32 {
    let mut max = 0;
    for child in &smali_node.children {
        max = max.max(max_offset_hint(child));
    }
    if let Some(offset_hint) = smali_node.offset_hint {
        max = max.max(offset_hint);
    }
    max
}

fn offset_or_stub(
    max_offset_len: usize, smali_node: &SmaliNode,
) -> SmaliToken {
    let raw = if let Some(offset_hint) = smali_node.offset_hint {
        format!("{:width$}", offset_hint, width = max_offset_len)
    } else {
        " ".repeat(max_offset_len)
    };
    SmaliToken::Other(raw.to_ref())
}

fn indent(indent_width: usize) -> SmaliToken {
    SmaliToken::Other(" ".repeat(indent_width).to_ref())
}
