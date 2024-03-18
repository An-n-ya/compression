#![allow(unused)]
use std::{fs::File, io::Write};

pub trait GraphViz {
    fn node_name(&self) -> String;
    fn node_attribute(&self) -> String;
    fn edge(&self) -> String;
    fn child(&self) -> Vec<Box<impl GraphViz>>;
    fn draw_to_file(&self, file_name: &str) {
        let mut file = File::create(file_name).unwrap();
        let script = self.draw_to_string();
        write!(file, "{script}").unwrap();
    }

    fn draw_to_string(&self) -> String {
        let mut node_attributes = vec![self.node_attribute()];
        let mut edges = vec![self.edge()];

        fn check(
            node: Box<impl GraphViz>,
            node_attributes: &mut Vec<String>,
            edges: &mut Vec<String>,
        ) {
            node_attributes.push(node.node_attribute());
            edges.push(node.edge());
            for sub_node in node.child() {
                check(sub_node, node_attributes, edges);
            }
        }

        for sub_node in self.child() {
            check(sub_node, &mut node_attributes, &mut edges);
        }

        let mut script = "graph {\n".to_string();
        let node_attributes = node_attributes.join("\n");
        script += &node_attributes;
        script.push('\n');
        let edges = edges.join("\n");
        script += &edges;
        script.push_str("\n}");
        script
    }
}
