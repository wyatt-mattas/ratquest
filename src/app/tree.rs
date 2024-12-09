use rat_tree_view::{Node, NodeValue, Tree};
use ratatui::style::{Color, Style};

use super::models::RequestType;
use super::state::App;

#[derive(Default, Clone)]
pub struct TreeNode {
    text: String,
    style: Option<Style>,
}

impl TreeNode {
    pub fn new(text: String) -> Self {
        Self { text, style: None }
    }

    pub fn with_style(text: String, style: Style) -> Self {
        Self {
            text,
            style: Some(style),
        }
    }
}

impl NodeValue for TreeNode {
    fn render_parts_iter(&self) -> impl Iterator<Item = (&str, Option<Style>)> {
        vec![(self.text.as_str(), self.style)].into_iter()
    }
}

impl App {
    pub fn build_tree(&self) -> Tree<TreeNode> {
        let mut root = Node::new("/".to_string(), TreeNode::new("API Groups".to_string()));

        for (group_name, requests) in &self.list {
            let mut group_node = Node::new(
                format!("group-{}", group_name),
                TreeNode::new(group_name.clone()),
            );

            for request in requests {
                let request_id = format!("request-{}-{}", group_name, request.name);

                let (symbol, style) = match request.request_type {
                    RequestType::GET => ("○", Style::default().fg(Color::Green)),
                    RequestType::POST => ("+", Style::default().fg(Color::Blue)),
                    RequestType::PUT => ("↺", Style::default().fg(Color::Yellow)),
                    RequestType::DELETE => ("-", Style::default().fg(Color::Red)),
                    RequestType::PATCH => ("~", Style::default().fg(Color::Magenta)),
                };

                let display_text = format!(
                    "{} {} {}",
                    symbol,
                    request.request_type.as_str(),
                    request.name
                );
                let request_node = Node::new(request_id, TreeNode::with_style(display_text, style));
                group_node.add_child(request_node);
            }

            root.add_child(group_node);
        }

        Tree::new(root)
    }
}
