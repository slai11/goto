use anyhow::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn pretty_print(db: &Vec<(&String, &String)>) -> Result<()> {
    let mut dummy = Node {
        next_level: Vec::new(),
        name: "".to_string(),
        val: None,
        position: 0,
    };

    // create tree to traverse
    for pair in db.iter() {
        let k = pair.0;
        let v = pair.1;

        let mut ptr = &mut dummy;

        let target_folder = v.split("/").last(); // really lazy, but its so fast anyway
        for folder in v.split("/") {
            // ignore the first empty split item
            if folder != "" {
                let val = if Some(folder) == target_folder {
                    Some(k.to_string())
                } else {
                    None
                };
                ptr.insert_if_absent(&folder, val);
                ptr = ptr.find(&folder).unwrap();
            }
        }
    }

    // dummy has no prefix and no sibling nodes,
    // hence we seed with empty string and 0.
    dummy.prettyprint("".to_string(), 0)
}

#[derive(Debug)]
struct Node {
    // Contains vec of child nodes
    next_level: Vec<Node>,

    // Name of folder
    name: String,

    // Some if there is an alias at this node
    val: Option<String>,

    // Position in parent node's next_level vec
    position: usize,
}

impl Node {
    // Returns mutable reference to child nodes with matching name
    fn find(&mut self, name: &str) -> Option<&mut Node> {
        self.next_level.iter_mut().find(|n| n.name == name)
    }

    // Creates a new node and push into back of vector if it does not exist
    fn insert_if_absent(&mut self, folder: &str, val: Option<String>) {
        if self.next_level.iter().find(|n| n.name == folder).is_none() {
            self.next_level.push(Node {
                next_level: Vec::new(),
                name: folder.to_string(),
                val: val,
                position: self.next_level.len() + 1,
            })
        }
    }

    // Pretty prints 1 line of content and initiates pretty print of all child nodes
    // Passes parent property of no. of children for child nodes to determine their relative
    // position.
    fn prettyprint(&self, prefix: String, node_fam_size: usize) -> Result<()> {
        // position determines t or l

        let indent = "  ";
        let has_next = self.position < node_fam_size;

        if self.name != "" {
            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;

            // print prefix depending on node's relative position
            let sym = if has_next { "├─" } else { "└─" };
            let pline = format!("{}{} ", prefix, sym);
            write!(&mut stdout, "{}", pline)?;

            // print in color only if folder is indexed
            match &self.val {
                None => {
                    writeln!(&mut stdout, "{}", &self.name)?;
                }
                Some(v) => {
                    if *v == *self.name {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                        writeln!(&mut stdout, "{}", &self.name)?;
                    } else {
                        // prints terminal folder name in white and adds its alias in blue
                        write!(&mut stdout, "{}", &self.name)?;
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                        writeln!(&mut stdout, " [{}]", &v)?;
                    }
                }
            }
        }

        let new_prefix = if has_next {
            prefix + "│" + indent
        } else {
            prefix + indent
        };

        // recursively print out child nodes
        for n in self.next_level.iter() {
            n.prettyprint(new_prefix.to_string(), self.next_level.len())?;
        }
        Ok(())
    }
}
