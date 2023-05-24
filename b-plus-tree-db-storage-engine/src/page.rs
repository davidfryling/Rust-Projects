// Page is a wrapper for a single page of memory providing some helpers for quick access
pub struct Page {
    data: Box<[u8; PAGE_SIZE]>,
}


/// Implement TryFrom<Box<Node>> for Page allowing for easier serialization of data from a Node to an on-disk formatted page
impl TryFrom<&Node> for Page {
    type Error = Error;
    fn try_from(node: &Node) -> Result<Page, Error> {
        let mut data: [u8; PAGE_SIZE] = [0x00; PAGE_SIZE];
        // is_root byte
        data[IS_ROOT_OFFSET] = node.is_root.to_byte();

        // node_type byte
        data[NODE_TYPE_OFFSET] = u8::from(&node.node_type);

        // parent offest
        if !node.is_root {
            match node.parent_offset {
                Some(Offset(parent_offset)) => data
                    [PARENT_POINTER_OFFSET..PARENT_POINTER_OFFSET + PARENT_POINTER_SIZE]
                    .clone_from_slice(&parent_offset.to_be_bytes()),
                // Expected an offset of an inner / leaf node.
                None => return Err(Error::UnexpectedError),
            };
        }

        match &node.node_type {
            NodeType::Internal(child_offsets, keys) => {
                data[INTERNAL_NODE_NUM_CHILDREN_OFFSET
                    ..INTERNAL_NODE_NUM_CHILDREN_OFFSET + INTERNAL_NODE_NUM_CHILDREN_SIZE]
                    .clone_from_slice(&child_offsets.len().to_be_bytes());

                let mut page_offset = INTERNAL_NODE_HEADER_SIZE;
                for Offset(child_offset) in child_offsets {
                    data[page_offset..page_offset + PTR_SIZE]
                        .clone_from_slice(&child_offset.to_be_bytes());
                    page_offset += PTR_SIZE;
                }

                for Key(key) in keys {
                    let key_bytes = key.as_bytes();
                    let mut raw_key: [u8; KEY_SIZE] = [0x00; KEY_SIZE];
                    if key_bytes.len() > KEY_SIZE {
                        return Err(Error::UnexpectedError);
                    } else {
                        for (i, byte) in key_bytes.iter().enumerate() {
                            raw_key[i] = *byte;
                        }
                    }
                    data[page_offset..page_offset + KEY_SIZE].clone_from_slice(&raw_key);
                    page_offset += KEY_SIZE
                }
            }
            NodeType::Leaf(kv_pairs) => {
                // num of pairs
                data[LEAF_NODE_NUM_PAIRS_OFFSET
                    ..LEAF_NODE_NUM_PAIRS_OFFSET + LEAF_NODE_NUM_PAIRS_SIZE]
                    .clone_from_slice(&kv_pairs.len().to_be_bytes());

                let mut page_offset = LEAF_NODE_HEADER_SIZE;
                for pair in kv_pairs {
                    let key_bytes = pair.key.as_bytes();
                    let mut raw_key: [u8; KEY_SIZE] = [0x00; KEY_SIZE];
                    if key_bytes.len() > KEY_SIZE {
                        return Err(Error::UnexpectedError);
                    } else {
                        for (i, byte) in key_bytes.iter().enumerate() {
                            raw_key[i] = *byte;
                        }
                    }
                    data[page_offset..page_offset + KEY_SIZE].clone_from_slice(&raw_key);
                    page_offset += KEY_SIZE;

                    let value_bytes = pair.value.as_bytes();
                    let mut raw_value: [u8; VALUE_SIZE] = [0x00; VALUE_SIZE];
                    if value_bytes.len() > VALUE_SIZE {
                        return Err(Error::UnexpectedError);
                    } else {
                        for (i, byte) in value_bytes.iter().enumerate() {
                            raw_value[i] = *byte;
                        }
                    }
                    data[page_offset..page_offset + VALUE_SIZE].clone_from_slice(&raw_value);
                    page_offset += VALUE_SIZE;
                }
            }
            NodeType::Unexpected => return Err(Error::UnexpectedError),
        }

        Ok(Page::new(data))
    }
}

// TODO: Add tests here...