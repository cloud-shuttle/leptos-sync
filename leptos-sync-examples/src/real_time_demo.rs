//! Real-time synchronization demo application

use leptos::prelude::*;
use leptos_sync_core::{
    storage::{IndexedDbStorage, LocalStorage},
    transport::{WebSocketTransport, HybridTransport},
    collection::{LocalFirstCollection, CollectionBuilder},
    crdt::ReplicaId,
    sync::SyncState,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TodoItem {
    id: String,
    text: String,
    completed: bool,
    created_at: String,
    last_modified: String,
    replica_id: String,
}

impl TodoItem {
    fn new(text: String, replica_id: String) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            completed: false,
            created_at: now.clone(),
            last_modified: now,
            replica_id,
        }
    }

    fn update(&mut self, text: String) {
        self.text = text;
        self.last_modified = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
        self.last_modified = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }
}

#[component]
fn RealTimeTodoApp() -> impl IntoView {
    let (todos, set_todos) = signal(Vec::new());
    let (new_todo_text, set_new_todo_text) = signal(String::new());
    let (sync_status, set_sync_status) = signal(String::new());
    let (sync_state, set_sync_state) = signal(SyncState::NotSynced);
    let (peer_count, set_peer_count) = signal(0);
    let (is_online, set_is_online) = signal(false);
    let (peers, set_peers) = signal(Vec::new());
    
    // Initialize collection with real-time sync
    let collection = create_memo(move |_| {
        let storage = IndexedDbStorage::new("leptos_sync_realtime".to_string(), "todos".to_string());
        let transport = HybridTransport::with_websocket("ws://localhost:8080/ws".to_string());
        
        CollectionBuilder::new(storage, transport)
            .with_auto_sync(true)
            .build()
    });
    
    // Load existing todos on mount
    create_effect(move |_| {
        let collection = collection.get();
        spawn_local(async move {
            if let Ok(keys) = collection.storage.keys().await {
                let mut loaded_todos = Vec::new();
                for key in keys {
                    if let Ok(Some(todo)) = collection.storage.get::<TodoItem>(&key).await {
                        loaded_todos.push(todo);
                    }
                }
                set_todos.set(loaded_todos);
                set_sync_status.set(format!("Loaded {} todos from storage", loaded_todos.len()));
            }
        });
    });
    
    // Start sync when component mounts
    create_effect(move |_| {
        let collection = collection.get();
        spawn_local(async move {
            if let Err(e) = collection.start_sync().await {
                set_sync_status.set(format!("Failed to start sync: {}", e));
            } else {
                set_sync_status.set("Real-time sync started!".to_string());
            }
        });
    });
    
    // Monitor sync state
    create_effect(move |_| {
        let collection = collection.get();
        spawn_local(async move {
            let info = collection.sync_info().await;
            set_sync_state.set(info.sync_state);
            set_peer_count.set(info.peer_count);
            set_is_online.set(info.is_online);
        });
    });
    
    // Monitor peers
    create_effect(move |_| {
        let collection = collection.get();
        spawn_local(async move {
            let peer_list: Vec<_> = collection.peers().await.collect();
            set_peers.set(peer_list.into_iter().map(|(id, info)| (id.0.to_string(), info)).collect());
        });
    });
    
    // Add new todo
    let add_todo = move |_| {
        let text = new_todo_text.get();
        if !text.is_empty() {
            let collection = collection.get();
            let replica_id = collection.replica_id().0.to_string();
            let todo = TodoItem::new(text, replica_id);
            
            spawn_local(async move {
                if let Err(e) = collection.insert(todo.id.clone(), todo.clone()).await {
                    set_sync_status.set(format!("Error saving todo: {}", e));
                } else {
                    set_sync_status.set("Todo saved and synced!".to_string());
                    // Reload todos
                    if let Ok(keys) = collection.storage.keys().await {
                        let mut loaded_todos = Vec::new();
                        for key in keys {
                            if let Ok(Some(todo)) = collection.storage.get::<TodoItem>(&key).await {
                                loaded_todos.push(todo);
                            }
                        }
                        set_todos.set(loaded_todos);
                    }
                }
            });
            
            set_new_todo_text.set(String::new());
        }
    };
    
    // Toggle todo completion
    let toggle_todo = move |todo_id: String| {
        let collection = collection.get();
        let todos_clone = todos.get();
        
        spawn_local(async move {
            if let Some(todo) = todos_clone.iter().find(|t| t.id == todo_id) {
                let mut updated_todo = todo.clone();
                updated_todo.toggle();
                
                if let Err(e) = collection.insert(&todo_id, &updated_todo).await {
                    set_sync_status.set(format!("Error updating todo: {}", e));
                } else {
                    set_sync_status.set("Todo updated and synced!".to_string());
                    // Reload todos
                    if let Ok(keys) = collection.storage.keys().await {
                        let mut loaded_todos = Vec::new();
                        for key in keys {
                            if let Ok(Some(todo)) = collection.storage.get::<TodoItem>(&key).await {
                                loaded_todos.push(todo);
                            }
                        }
                        set_todos.set(loaded_todos);
                    }
                }
            }
        });
    };
    
    // Delete todo
    let delete_todo = move |todo_id: String| {
        let collection = collection.get();
        
        spawn_local(async move {
            if let Err(e) = collection.remove(todo_id.clone()).await {
                set_sync_status.set(format!("Error deleting todo: {}", e));
            } else {
                set_sync_status.set("Todo deleted and synced!".to_string());
                // Reload todos
                if let Ok(keys) = collection.storage.keys().await {
                    let mut loaded_todos = Vec::new();
                    for key in keys {
                        if let Ok(Some(todo)) = collection.storage.get::<TodoItem>(&key).await {
                            loaded_todos.push(todo);
                        }
                    }
                    set_todos.set(loaded_todos);
                }
            }
        });
    };
    
    // Force sync
    let force_sync = move |_| {
        let collection = collection.get();
        
        spawn_local(async move {
            if let Err(e) = collection.force_sync().await {
                set_sync_status.set(format!("Force sync failed: {}", e));
            } else {
                set_sync_status.set("Force sync completed!".to_string());
            }
        });
    };
    
    // Start/stop sync
    let toggle_sync = move |_| {
        let collection = collection.get();
        let current_state = sync_state.get();
        
        spawn_local(async move {
            let result = if matches!(current_state, SyncState::Syncing) {
                collection.stop_sync().await
            } else {
                collection.start_sync().await
            };
            
            match result {
                Ok(()) => {
                    let new_state = if matches!(current_state, SyncState::Syncing) {
                        SyncState::NotSynced
                    } else {
                        SyncState::Syncing
                    };
                    set_sync_state.set(new_state);
                }
                Err(e) => {
                    set_sync_status.set(format!("Sync toggle failed: {}", e));
                }
            }
        });
    };
    
    view! {
        <div style="max-width: 1200px; margin: 0 auto; padding: 20px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
            <h1 style="color: #333; text-align: center; margin-bottom: 30px;">
                "🚀 Leptos-Sync Real-Time Demo"
            </h1>
            
            // Real-time sync status
            <div style="background: #e3f2fd; border: 1px solid #2196f3; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                <h2 style="color: #1976d2; margin-top: 0;">"Real-Time Sync Status"</h2>
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px;">
                    <div class="status-card">
                        <strong>"Sync State:"</strong>
                        <span class=move || {
                            match sync_state.get() {
                                SyncState::NotSynced => "⏸️ Not Syncing",
                                SyncState::Syncing => "🔄 Syncing...",
                                SyncState::Synced => "✅ Synchronized",
                                SyncState::Failed(e) => format!("❌ Failed: {}", e),
                                SyncState::ResolvingConflicts => "⚠️ Resolving Conflicts",
                                SyncState::Offline => "📡 Offline",
                            }
                        }></span>
                    </div>
                    <div class="status-card">
                        <strong>"Connection:"</strong>
                        <span class=move || if is_online.get() { "🟢 Online" } else { "🔴 Offline" }></span>
                    </div>
                    <div class="status-card">
                        <strong>"Peers:"</strong>
                        <span>{peer_count}</span>
                    </div>
                    <div class="status-card">
                        <strong>"Status:"</strong>
                        <span>{sync_status}</span>
                    </div>
                </div>
                
                <div style="margin-top: 15px; display: flex; gap: 10px; flex-wrap: wrap;">
                    <button
                        class="btn btn-primary"
                        on:click=toggle_sync
                    >
                        {move || if matches!(sync_state.get(), SyncState::Syncing) { "Stop Sync" } else { "Start Sync" }}
                    </button>
                    <button
                        class="btn btn-secondary"
                        on:click=force_sync
                    >
                        "Force Sync"
                    </button>
                </div>
            </div>
            
            // Peer list
            {move || {
                let peer_list = peers.get();
                if !peer_list.is_empty() {
                    view! {
                        <div style="background: #f3e5f5; border: 1px solid #9c27b0; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                            <h3 style="color: #7b1fa2; margin-top: 0;">"Connected Peers"</h3>
                            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 10px;">
                                {peer_list.into_iter().map(|(id, info)| {
                                    view! {
                                        <div style="background: white; border: 1px solid #e1bee7; border-radius: 4px; padding: 10px;">
                                            <div style="font-weight: bold; color: #7b1fa2;">{id}</div>
                                            <div style="font-size: 0.9em; color: #666;">
                                                "Last seen: {info.last_seen.format("%H:%M:%S")}"
                                            </div>
                                            <div style="font-size: 0.9em; color: #666;">
                                                if info.is_online { "🟢 Online" } else { "🔴 Offline" }
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                } else {
                    view! {
                        <div style="background: #fff3e0; border: 1px solid #ff9800; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                            <h3 style="color: #e65100; margin-top: 0;">"No Peers Connected"</h3>
                            <p style="color: #e65100; margin-bottom: 0;">
                                "Start the WebSocket server to see real-time synchronization in action!"
                            </p>
                        </div>
                    }
                }
            }}
            
            // Add new todo form
            <div style="background: #f5f5f5; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                <h3 style="margin-top: 0;">"Add New Todo"</h3>
                <div style="display: flex; gap: 10px;">
                    <input
                        type="text"
                        placeholder="Enter todo text..."
                        style="flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 4px;"
                        on:input=move |ev| {
                            set_new_todo_text.set(event_target_value(&ev));
                        }
                        prop:value=new_todo_text
                    />
                    <button
                        on:click=add_todo
                        style="padding: 10px 20px; background: #4caf50; color: white; border: none; border-radius: 4px; cursor: pointer;"
                    >
                        "Add Todo"
                    </button>
                </div>
            </div>
            
            // Todo list
            <div style="background: #ffffff; border: 1px solid #ddd; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
                <h3 style="margin: 0;">"Real-Time Todo List"</h3>
                {move || {
                    let todos_list = todos.get();
                    if todos_list.is_empty() {
                        view! {
                            <p style="text-align: center; color: #666; font-style: italic;">
                                "No todos yet. Add one above!"
                            </p>
                        }
                    } else {
                        view! {
                            <div style="display: grid; gap: 10px;">
                                {todos_list.into_iter().map(|todo| {
                                    let todo_id = todo.id.clone();
                                    view! {
                                        <div
                                            style="display: flex; justify-content: space-between; align-items: center; padding: 15px; border: 1px solid #eee; border-radius: 4px; background: #fafafa;"
                                        >
                                            <div style="display: flex; align-items: center; gap: 10px; flex: 1;">
                                                <input
                                                    type="checkbox"
                                                    checked=todo.completed
                                                    on:change=move |_| toggle_todo(todo_id.clone())
                                                />
                                                <div style="flex: 1;">
                                                    <span style=move || if todo.completed { "text-decoration: line-through; color: #888;" } else { "" }>
                                                        {todo.text}
                                                    </span>
                                                    <div style="font-size: 0.8em; color: #666; margin-top: 5px;">
                                                        <div>"Created: {todo.created_at}"</div>
                                                        <div>"Modified: {todo.last_modified}"</div>
                                                        <div>"Replica: {todo.replica_id}"</div>
                                                    </div>
                                                </div>
                                            </div>
                                            <button
                                                on:click=move |_| delete_todo(todo_id.clone())
                                                style="padding: 4px 8px; background: #ff9800; color: white; border: none; border-radius: 4px; cursor: pointer;"
                                            >
                                                "Delete"
                                            </button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }
                    }
                }}
            </div>
            
            // Instructions
            <div style="background: #e8f5e8; border: 1px solid #4caf50; border-radius: 8px; padding: 20px;">
                <h3 style="color: #2e7d32; margin-top: 0;">"🧪 Testing Real-Time Sync"</h3>
                <ol style="color: #2e7d32; margin: 10px 0;">
                    <li>"Start the WebSocket server: <code>cargo run --example websocket-server</code>"</li>
                    <li>"Open this page in multiple browser tabs/windows"</li>
                    <li>"Add, edit, or delete todos in one tab"</li>
                    <li>"Watch them appear in real-time in other tabs!"</li>
                    <li>"Check the sync status and peer information above"</li>
                </ol>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Starting Leptos-Sync Real-Time Demo".into());
    
    mount_to_body(RealTimeTodoApp)
}
