// No, not that kind of docking :p
#![allow(unused)]

use std::os::raw::c_char;

use imgui::{Direction, sys};

/// Rust version of  the Dear ImGUI struct DockNode
#[repr(C)]
pub struct DockNode {
    id: u32,
}

impl DockNode {
    /// Creates a new [DockNode] with a given [id]
    fn new(id: u32) -> Self {
        Self { id }
    }

    /// Returns whether the current [DockNode] is split (`true`), or if it contains a single child window (`false`)
    pub fn is_split(&self) -> bool {
        unsafe {
            let node = sys::igDockBuilderGetNode(self.id);
            // I assume we have nothing docked for this ID, or the ID is invalid?
            if node.is_null() {
                false
            } else {
                sys::ImGuiDockNode_IsSplitNode(node)
            }
        }
    }
    /// Dock window into this dockspace
    #[doc(alias = "DockBuilder::DockWindow")]
    pub fn dock_window(&self, window: &str) {
        let window = imgui::ImString::from(window.to_string());
        unsafe { sys::igDockBuilderDockWindow(window.as_ptr(), self.id) }
    }

    /// Splits the current node along a direction
    ///
    /// # Params
    /// * [split_side] - The side to create the main split in. This is the side that is considered the "first" split
    /// * [size_ratio] - ?How the split is shared between the two resulting child nodes (how equal their new area is)
    /// * [child_a] - Closure to build the first child (see [split_side]).
    /// * [child_b] - Closure to build the second child (opposite of [child_a])
    #[doc(alias = "DockBuilder::SplitNode")]
    pub fn split<BuildChildA, BuildChildB>(&self, split_side: Direction, size_ratio: f32, child_a: BuildChildA, child_b: BuildChildB)
                                           where
                                               BuildChildA: FnOnce(DockNode),
                                               BuildChildB: FnOnce(DockNode),
    {
        if self.is_split() {
            // Can't split an already split node (need to split the
            // node within)
            return;
        }

        let mut out_id_at_dir: sys::ImGuiID = 0;
        let mut out_id_at_opposite_dir: sys::ImGuiID = 0;
        unsafe {
            sys::igDockBuilderSplitNode(
                self.id,
                split_side as i32,
                size_ratio,
                &mut out_id_at_dir,
                &mut out_id_at_opposite_dir,
            );
        }

        child_a(DockNode::new(out_id_at_dir));
        child_b(DockNode::new(out_id_at_opposite_dir));
    }
}

//TODO: Find out what's necessary in these files. May be able to remove UiDockingArea struct completely
/// # Docking
pub struct UiDockingArea {}

impl UiDockingArea {
    /// Wrapper around Dear ImGUI's `IsWindowDocked` function
    ///
    /// (Unsure) Returns whether the last window to be created is currently docked
    #[doc(alias = "IsWindowDocked")]
    pub fn is_window_docked(&self) -> bool {
        unsafe { sys::igIsWindowDocked() }
    }

    /// Create dockspace with given label. Returns a handle to the
    /// dockspace which can be used to, say, programmatically split or
    /// dock windows into it
    #[doc(alias = "DockSpace")]
    pub fn dockspace(&self, label: &str) -> DockNode {
        let label = imgui::ImString::from(label.to_string());
        unsafe {
            let id = sys::igGetIDStr(label.as_ptr() as *const c_char);
            sys::igDockSpace(
                id,
                [0.0, 0.0].into(),
                0,
                ::std::ptr::null::<sys::ImGuiWindowClass>(),
            );
            DockNode { id }
        }
    }

    /// (Unsure) Moves the dockspace to be over the main viewport (that seems to be hardcoded in)
    #[doc(alias = "DockSpaceOverViewport")]
    pub fn dockspace_over_viewport(&self) {
        unsafe {
            sys::igDockSpaceOverViewport(
                sys::igGetMainViewport(),
                0,
                ::std::ptr::null::<sys::ImGuiWindowClass>(),
            );
        }
    }
}