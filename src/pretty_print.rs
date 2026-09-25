use crate::db::GotoFile;
use anyhow::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// Output is captured by the `gt` shell function before being echoed, so stdout
// is never a tty here and colors must be forced on.
fn stdout() -> StandardStream {
    StandardStream::stdout(ColorChoice::Always)
}

fn set_fg(stdout: &mut StandardStream, color: Color) -> Result<()> {
    stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
    Ok(())
}

pub fn pretty_print_tree(db: &[(&String, &GotoFile)]) -> Result<()> {
    let mut root = Node::default();
    for (alias, entry) in db {
        let mut node = &mut root;
        for folder in entry.path.split('/').filter(|f| !f.is_empty()) {
            node = node.child(folder);
        }
        node.alias = Some(alias.to_string());
    }

    let mut stdout = stdout();
    for (i, child) in root.children.iter().enumerate() {
        child.print(&mut stdout, "   ", i + 1 == root.children.len())?;
    }
    Ok(())
}

pub fn pretty_print_jumpsites(sites: &[GotoFile]) -> Result<()> {
    let mut stdout = stdout();
    set_fg(&mut stdout, Color::White)?;
    writeln!(stdout, "Listing all jump sites")?;
    writeln!(stdout)?;

    for (i, site) in sites.iter().enumerate() {
        let (parent, name) = site.path.rsplit_once('/').unwrap_or(("", &site.path));
        write!(stdout, "[")?;
        set_fg(&mut stdout, Color::Blue)?;
        write!(stdout, "{}", i + 1)?;
        set_fg(&mut stdout, Color::White)?;
        write!(stdout, "]")?;

        // grey + white for contrast
        set_fg(&mut stdout, Color::Rgb(128, 128, 128))?;
        write!(stdout, " {}/", parent)?;
        set_fg(&mut stdout, Color::White)?;
        writeln!(stdout, "{}", name)?;
    }
    Ok(())
}

#[derive(Debug, Default)]
struct Node {
    // Name of folder
    name: String,

    // Some if this folder is indexed
    alias: Option<String>,

    children: Vec<Node>,
}

impl Node {
    // Returns the child with a matching name, creating it at the back if absent
    fn child(&mut self, name: &str) -> &mut Node {
        let idx = match self.children.iter().position(|n| n.name == name) {
            Some(idx) => idx,
            None => {
                self.children.push(Node {
                    name: name.to_string(),
                    ..Node::default()
                });
                self.children.len() - 1
            }
        };
        &mut self.children[idx]
    }

    // Prints this node's line, then recurses into its children
    fn print(&self, stdout: &mut StandardStream, prefix: &str, is_last: bool) -> Result<()> {
        let indent = "  ";

        set_fg(stdout, Color::White)?;
        write!(stdout, "{}{} ", prefix, if is_last { "└─" } else { "├─" })?;

        // print in color only if folder is indexed
        match &self.alias {
            None => writeln!(stdout, "{}", self.name)?,
            Some(alias) if *alias == self.name => {
                set_fg(stdout, Color::Blue)?;
                writeln!(stdout, "{}", self.name)?;
            }
            Some(alias) => {
                // prints terminal folder name in white and adds its alias in blue
                write!(stdout, "{}", self.name)?;
                set_fg(stdout, Color::Blue)?;
                writeln!(stdout, " [{}]", alias)?;
            }
        }

        let child_prefix = format!("{}{}{}", prefix, if is_last { " " } else { "│" }, indent);
        for (i, child) in self.children.iter().enumerate() {
            child.print(stdout, &child_prefix, i + 1 == self.children.len())?;
        }
        Ok(())
    }
}
