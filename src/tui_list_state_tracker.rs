
// Source: https://docs.rs/tui/latest/tui/widgets/trait.StatefulWidget.html#examples
// Create a ListStateTracker for each pick list we want to track.

use ratatui::{text::Span, widgets::{ListItem, ListState}};

// TODO: think of a better name than ListStateTracker
pub struct ListStateTracker {
    // `items` is the state managed by your application.
    pub items: Vec<String>,

    // `state` is the state that can be modified by the UI. It stores the index of the selected
    // item as well as the offset computed during the previous draw call (used to implement
    // natural scrolling).
    pub state: ListState
}

impl ListStateTracker {
    fn new(items: Vec<String>) -> ListStateTracker {
        // sort the items
        let mut items = items;
        items.sort_unstable();

        let mut ret = ListStateTracker {
            items: items,
            state: ListState::default(),
        };
        ret.reset_selection();
        ret
    }

    pub fn default() -> ListStateTracker {
        ListStateTracker {
            items: Vec::new(),
            state: ListState::default(),
        }
    }

    /// Update the items in the list. Note that items are sorted before being stored.
    pub fn update_items(&mut self, items: Vec<String>) {
        // sort the items
        let mut items = items;
        items.sort_unstable();

        // check if the items have changed
        if self.items != items {
            // eprintln!("Items have changed, updating items. old={:?}, new_unsort={:?}, new_sort={:?}", self.items, items, items.clone());
            self.items = items;
            self.reset_selection();
        }
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        // sort the items
        let mut items_storted = items;
        items_storted.sort_unstable();

        self.items = items_storted;
        self.reset_selection();
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn next(&mut self) {
        if self.items.is_empty() {
            self.unselect();
            return;
        }
        let new_idx = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(new_idx));
    }

    // Select the previous item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn previous(&mut self) {
        if self.items.is_empty() {
            self.unselect();
            return;
        }
        let new_idx = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(new_idx));
    }

    // Select the first item, if there are any items. Otherwise, unselect the current selection.
    pub fn reset_selection(&mut self) {
        if self.items.is_empty() {
            self.unselect();
        }
        else {
            self.state.select(Some(0));
        }
    }

    // Unselect the currently selected item if any. The implementation of `ListState` makes
    // sure that the stored offset is also reset.
    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    // Get the selected item.
    pub fn get_selected(&self) -> Option<String> {
        match self.state.selected() {
            Some(i) => Some(self.items[i].clone()),
            None => None,
        }
    }

    // TODO: best to refactor this to here, but can't figure out borrowing right now
    // pub fn get_as_list_items(self) -> Vec<ListItem> {
    //     self.items.iter().map(|item| {
    //         ListItem::new(Span::raw(format!("{}", item)))
    //     }).collect()
    // }

}

