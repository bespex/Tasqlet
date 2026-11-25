//! Search for the interface (docs/KONZEPT.md 6.4, plan 1.10): the matching is the core's, this
//! only hands the hits over in the shape of the snapshot.

use serde_json::{Value, json};
use tasqlet_core::search;
use tauri::{AppHandle, Manager};

use crate::data::{Data, item_json};

/// Most hits shown; a longer list is no use in a popover.
const LIMIT: usize = 50;
/// How many finished items the search shows without a query.
const RECENT: usize = 10;

/// The items matching `query`, best first; without a query the recently finished ones (`recent`).
#[tauri::command]
pub fn search(app: AppHandle, query: String) -> Value {
    let store = app.state::<Data>().store();
    let recent = search::terms(&query).is_empty();
    let hits = if recent {
        search::recently_done(&store, RECENT)
    } else {
        search::search(&store, &query, LIMIT)
    };
    json!({
        "recent": recent,
        "items": hits.into_iter().map(item_json).collect::<Vec<_>>(),
    })
}
