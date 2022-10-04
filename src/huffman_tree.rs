struct Node {
    parent: Option<usize>,
    child1: Option<usize>,
    child2: Option<usize>,

    weight: u32,
    character: Option<char>,
}

pub struct Tree {
    nodes: Vec<Node>,
}
impl Tree {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn new_from_string(s: &String) -> Self {
        let char_count = Tree::count_characters_in_string(s);

        let mut tree: Tree = Tree::new();
        for (c, count) in char_count {
            tree.add_node(count, c);
        }

        tree.add_node(1, '\0');
        tree.connect_all();

        tree
    }

    fn count_characters_in_string(s: &String) -> std::collections::HashMap<char, u32> {
        let mut char_count: std::collections::HashMap<char, u32> = std::collections::HashMap::new();
    
        for c in s.chars() {
            char_count
                .entry(c)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    
        char_count
    }

    fn add_node(&mut self, weight: u32, character: char) {
        self.nodes.push(Node {
            parent: None,
            child1: None,
            child2: None,
            weight,
            character: Some(character),
        });
    }

    fn connect_nodes(&mut self, node1: usize, node2: usize) {
        let parent_index = self.nodes.len();

        self.nodes.push(Node {
            parent: None,
            child1: Some(node1),
            child2: Some(node2),
            weight: self.nodes[node1].weight + self.nodes[node2].weight,
            character: None,
        });

        self.nodes[node1].parent = Some(parent_index);
        self.nodes[node2].parent = Some(parent_index);
    }

    fn find_two_smallest(&self) -> (usize, usize) {
        let mut smallest: u32 = u32::max_value();
        let mut smallest2: u32 = u32::max_value();
        let mut smallest_index: usize = 0;
        let mut smallest2_index: usize = 0;

        for i in 0..self.nodes.len() {
            if self.nodes[i].parent.is_some() {
                continue;
            }

            if self.nodes[i].weight < smallest {
                smallest2 = smallest;
                smallest2_index = smallest_index;
                smallest = self.nodes[i].weight;
                smallest_index = i;
                continue;
            }
            if self.nodes[i].weight < smallest2 {
                smallest2 = self.nodes[i].weight;
                smallest2_index = i;
            }
        }

        (smallest_index, smallest2_index)
    }

    fn connect_all(&mut self) {
        let mut whole_weight: u32 = 0;
        for node in &self.nodes {
            whole_weight += node.weight;
        }

        loop {
            let (smallest1, smallest2) = self.find_two_smallest();
            self.connect_nodes(smallest1, smallest2);

            let new_node_weight = self.nodes[self.nodes.len() - 1].weight;
            if new_node_weight >= whole_weight {
                break;
            }
        }
    }

    pub fn generate_list(&self, list: &mut Vec<(char, String)>, code: String) {
        let mut current_node: usize = self.nodes.len() - 1;
        for dir in code.chars() {
            match dir {
                '0' => current_node = self.nodes[current_node].child1.unwrap(),
                '1' => current_node = self.nodes[current_node].child2.unwrap(),
                _ => {}
            }
        }
    
        if self.nodes[current_node].character.is_some() {
            list.push((self.nodes[current_node].character.unwrap(), code));
            return;
        }
    
        self.generate_list(list, code.clone() + "0");
        self.generate_list(list, code.clone() + "1");
    }
}