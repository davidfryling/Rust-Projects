// Node represents a node in the BTree occupied by a single page in memory
pub struct Node {
    pub node_type: NodeType,
    pub is_root: bool,
    pub parent_offeset: Option<Offset>,
}

// NodeType represents different node types in the BTree
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NodeType {
    // Internal nodes contain a vecore of pointers to their children and a vector of keys
    Internal(Vec<Offset>, Vec<Key>),

    // Leaf nodes contain a vector of keys and values
    Leaf(Vec<KeyValuePair>),

    Unexpected,
}

// vvv... LEFT OFF HERE ...vvv

// Implement TryFrom<Page> for Node allowing for easier deserialization of data from a Page
impl TryFrom<Page> for Node {
    type Error = Error;
    fn try_from(page: Page) -> Result<Node, Error> {
        let raw = page.get_data();
        let node_type = NodeType::from(raw[NODE_TYPE_OFFSET]);
        let is_root = raw[IS_ROOT_OFFSET].from_byte();
        let parent_offset: Option<Offset>;
        if is_root {
            parent_offset = None;
        } else {
            parent_offset = Some(Offset(page.get_value_from_offset(PARENT_POINTER_OFFSET)?));
        }

        match node_type {
            NodeType::Internal(mut children, mut keys) => {
                let num_children = page.get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let mut offset = INTERNAL_NODE_HEADER_SIZE;
                for _i in 1..=num_children {
                    let child_offset = page.get_value_from_offset(offset)?;
                    children.push(Offset(child_offset));
                    offset += PTR_SIZE;
                }

                // Number of keys is always one less than the number of children (i.e. branching factor)
                for _i in 1..num_children {
                    let key_raw = page.get_ptr_from_offset(offset, KEY_SIZE);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += KEY_SIZE;
                    // Trim leading or trailing zeros.
                    keys.push(Key(key.trim_matches(char::from(0)).to_string()));
                }
                Ok(Node::new(
                    NodeType::Internal(children, keys),
                    is_root,
                    parent_offset,
                ))
            }

            NodeType::Leaf(mut pairs) => {
                let mut offset = LEAF_NODE_NUM_PAIRS_OFFSET;
                let num_keys_val_pairs = page.get_value_from_offset(offset)?;
                offset = LEAF_NODE_HEADER_SIZE;

                for _i in 0..num_keys_val_pairs {
                    let key_raw = page.get_ptr_from_offset(offset, KEY_SIZE);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += KEY_SIZE;

                    let value_raw = page.get_ptr_from_offset(offset, VALUE_SIZE);
                    let value = match str::from_utf8(value_raw) {
                        Ok(val) => val,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += VALUE_SIZE;

                    // Trim leading or trailing zeros.
                    pairs.push(KeyValuePair::new(
                        key.trim_matches(char::from(0)).to_string(),
                        value.trim_matches(char::from(0)).to_string(),
                    ))
                }
                Ok(Node::new(NodeType::Leaf(pairs), is_root, parent_offset))
            }

            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }
}

// TODO: Add tests here...