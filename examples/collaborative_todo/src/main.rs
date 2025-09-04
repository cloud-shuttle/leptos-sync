use leptos::*;
use leptos_sync_core::{
    collection::LocalFirstCollection,
    crdt::{LwwMap, LwwRegister, ReplicaId},
    storage::Storage,
    transport::memory::InMemoryTransport,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TodoItem {
    id: String,
    text: String,
    completed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TodoItem {
    fn new(text: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            completed: false,
            created_at: now,
            updated_at: now,
        }
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
        self.updated_at = chrono::Utc::now();
    }

    fn update_text(&mut self, new_text: String) {
        self.text = new_text;
        self.updated_at = chrono::Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TodoList {
    items: HashMap<String, TodoItem>,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TodoList {
    fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            items: HashMap::new(),
            name,
            created_at: now,
            updated_at: now,
        }
    }

    fn add_item(&mut self, item: TodoItem) {
        self.items.insert(item.id.clone(), item);
        self.updated_at = chrono::Utc::now();
    }

    fn remove_item(&mut self, id: &str) {
        self.items.remove(id);
        self.updated_at = chrono::Utc::now();
    }

    fn get_item(&self, id: &str) -> Option<&TodoItem> {
        self.items.get(id)
    }

    fn get_all_items(&self) -> Vec<&TodoItem> {
        self.items.values().collect()
    }

    fn get_completed_count(&self) -> usize {
        self.items.values().filter(|item| item.completed).count()
    }

    fn get_pending_count(&self) -> usize {
        self.items.values().filter(|item| !item.completed).count()
    }
}

// ============================================================================
// CRDT Wrappers
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TodoItemCRDT {
    item: LwwRegister<TodoItem>,
    metadata: LwwMap<String, String>,
}

impl TodoItemCRDT {
    fn new(item: TodoItem, replica_id: ReplicaId) -> Self {
        let mut metadata = LwwMap::new();
        metadata.insert("replica_id".to_string(), replica_id.0.to_string(), replica_id);
        
        Self {
            item: LwwRegister::new(item, replica_id),
            metadata,
        }
    }

    fn get_item(&self) -> &TodoItem {
        self.item.value()
    }

    fn update_item(&mut self, new_item: TodoItem, replica_id: ReplicaId) {
        self.item = LwwRegister::new(new_item, replica_id);
        self.metadata.insert("updated_at".to_string(), chrono::Utc::now().to_rfc3339(), replica_id);
    }
}

// ============================================================================
// Main App Component
// ============================================================================

#[component]
fn CollaborativeTodoApp() -> impl IntoView {
    let (todo_list, set_todo_list) = create_signal(TodoList::new("My Collaborative Todo List".to_string()));
    let (new_todo_text, set_new_todo_text) = create_signal(String::new());
    let (replica_id, _) = create_signal(ReplicaId::default());
    
    // Initialize sync collection
    let storage = Storage::memory();
    let transport = InMemoryTransport::new();
    let collection = LocalFirstCollection::<TodoItemCRDT, _>::new(storage, transport);

    // Add new todo item
    let add_todo = move |_| {
        let text = new_todo_text.get();
        if !text.trim().is_empty() {
            let item = TodoItem::new(text);
            let todo_crdt = TodoItemCRDT::new(item, replica_id.get());
            
            // Add to local state
            set_todo_list.update(|list| list.add_item(todo_crdt.get_item().clone()));
            
            // Add to sync collection
            spawn_local(async move {
                let _ = collection.insert(&todo_crdt.get_item().id, &todo_crdt).await;
            });
            
            set_new_todo_text.set(String::new());
        }
    };

    // Toggle todo completion
    let toggle_todo = move |id: String| {
        set_todo_list.update(|list| {
            if let Some(item) = list.get_item(&id) {
                let mut updated_item = item.clone();
                updated_item.toggle();
                list.add_item(updated_item);
            }
        });
    };

    // Remove todo item
    let remove_todo = move |id: String| {
        set_todo_list.update(|list| list.remove_item(&id));
    };

    // Update todo text
    let update_todo_text = move |id: String, new_text: String| {
        set_todo_list.update(|list| {
            if let Some(item) = list.get_item(&id) {
                let mut updated_item = item.clone();
                updated_item.update_text(new_text);
                list.add_item(updated_item);
            }
        });
    };

    view! {
        <div class="min-h-screen bg-gray-100 py-8">
            <div class="max-w-4xl mx-auto px-4">
                <h1 class="text-4xl font-bold text-center text-gray-800 mb-8">
                    "Collaborative Todo App"
                </h1>
                
                // Add new todo form
                <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                    <div class="flex gap-4">
                        <input
                            type="text"
                            placeholder="Add a new todo..."
                            class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value={new_todo_text.get()}
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_new_todo_text.set(value);
                            }
                        />
                        <button
                            class="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:click=add_todo
                        >
                            "Add Todo"
                        </button>
                    </div>
                </div>

                // Todo list
                <div class="bg-white rounded-lg shadow-md p-6">
                    <div class="flex justify-between items-center mb-6">
                        <h2 class="text-2xl font-semibold text-gray-800">
                            {move || todo_list.get().name}
                        </h2>
                        <div class="flex gap-4 text-sm text-gray-600">
                            <span>
                                "Pending: " {move || todo_list.get().get_pending_count()}
                            </span>
                            <span>
                                "Completed: " {move || todo_list.get().get_completed_count()}
                            </span>
                        </div>
                    </div>

                    <div class="space-y-3">
                        {move || {
                            let items = todo_list.get().get_all_items();
                            if items.is_empty() {
                                view! {
                                    <div class="text-center text-gray-500 py-8">
                                        "No todos yet. Add one above!"
                                    </div>
                                }
                            } else {
                                items.into_iter().map(|item| {
                                    let id = item.id.clone();
                                    view! {
                                        <div class="flex items-center gap-3 p-4 border border-gray-200 rounded-lg hover:bg-gray-50">
                                            <input
                                                type="checkbox"
                                                checked={item.completed}
                                                on:change=move |_| toggle_todo(id.clone())}
                                                class="w-5 h-5 text-blue-600 rounded focus:ring-blue-500"
                                            />
                                            <span class={move || if item.completed { "line-through text-gray-500" } else { "text-gray-800" }}>
                                                {item.text}
                                            </span>
                                            <div class="ml-auto flex gap-2">
                                                <button
                                                    class="px-3 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600"
                                                    on:click=move |_| remove_todo(id.clone())}
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()
                            }
                        }}
                    </div>
                </div>

                // CRDT Info
                <div class="bg-white rounded-lg shadow-md p-6 mt-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">"CRDT Information"</h3>
                    <div class="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <span class="font-medium">"Replica ID: "</span>
                            <span class="font-mono">{move || replica_id.get().0}</span>
                        </div>
                        <div>
                            <span class="font-medium">"Total Items: "</span>
                            <span>{move || todo_list.get().items.len()}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Main Function
// ============================================================================

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <CollaborativeTodoApp/> });
}
