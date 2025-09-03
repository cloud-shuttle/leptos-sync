//! Sync status indicator component

use leptos::*;
use leptos::prelude::*;

#[component]
pub fn SyncStatusIndicator() -> impl IntoView {
    view! {
        <div>
            <span>"Sync Status: Connected"</span>
        </div>
    }
}
