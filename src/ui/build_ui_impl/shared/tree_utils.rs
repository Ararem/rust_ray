use imgui::{TreeNodeFlags, TreeNodeId, TreeNodeToken};

/// Creates a tree node, in such a way that custom text can be used instead of a simple label
///
/// This creates a tree node with an empty ("") label, configures it to take up the full width available to it, and then resets the cursor position to just after the node's dowpdown arrow
/// This allows you to make calls using [imgui::Ui::same_line()] and [imgui::Ui::text_colored()] to make a much nicer label.
///
/// # Example:
/// ```
/// let maybe_tree_node = tree_node_with_custom_text(ui, TreeNodeId::Ptr(ip)); // Use the frame's instruction pointer as the tree node's ID, so it's unique
//  ui.text_colored(colours.value.symbol, "[");
//  ui.same_line_with_spacing(0.0, 0.0);
//  ui.text_colored(colours.value.number, depth.to_string());
//  ui.same_line_with_spacing(0.0, 0.0);
//  ui.text_colored(colours.value.symbol, "]: ");
//  ui.same_line_with_spacing(0.0, 0.0);
//  ui.text_colored(colours.value.name, metadata.name());
//
//  let tree_node = match maybe_tree_node {
//      None => {
//          // This specific span's node is closed
//          return;
//      }
//      Some(node) => node,
//  };
/// ```
#[must_use]
pub fn tree_node_with_custom_text<Id, TId>(ui: &imgui::Ui, id: Id) -> Option<TreeNodeToken>
where
    Id: Into<TreeNodeId<TId>>,
    TId: AsRef<str>,
{
    // If the node is expanded, then we get to see all the juicy information
    // Empty label is important to make sure we don't display anything, will customise further down
    // Also important is the SPAN_AVAIL_WIDTH flag, so it fills the "empty" space on the right of the arrow
    // This means the hitbox is extended, but if we use [same_line()] we can overlap it
    // Neat!

    // Calculate the indented cursor position, so we can find where we should start putting the text
    // So that it appears in-line with the tree node
    // Do this before creating the node because the node fills the remaining space
    // Originally this used [ui.indent()], but this doesn't scale with larger font sizes (the dropwdown arrow grows but the indent is the same, so the text starts overlapping the arrow)
    // So we do a fake indent using the font size, which works much better
    ui.indent_by(ui.current_font_size());
    let cursor_pos = ui.cursor_pos();
    ui.unindent_by(ui.current_font_size());

    let maybe_tree_node = ui
        .tree_node_config(id)
        .label::<&str, &str>("")
        .flags(TreeNodeFlags::SPAN_AVAIL_WIDTH)
        .push();

    // Fancy colours are always better than simple ones right?
    ui.same_line_with_pos(cursor_pos[0]); // Reset pos to just after tree node
    maybe_tree_node
}
