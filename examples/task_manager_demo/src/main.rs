use leptos::prelude::*;
use task_manager_demo::task_manager::{TaskManager, Task, TaskStatus, TaskPriority};
use leptos_sync_core::crdt::Mergeable;
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[component]
pub fn TaskManagerDemo() -> impl IntoView {
    let user_id = Uuid::new_v4();
    let (manager, set_manager) = signal(TaskManager::new(user_id));
    let (new_task_title, set_new_task_title) = signal(String::new());
    let (new_task_description, set_new_task_description) = signal(String::new());
    let (new_task_priority, set_new_task_priority) = signal(TaskPriority::Medium);
    let (filter_status, set_filter_status) = signal(Option::<TaskStatus>::None);
    let (filter_priority, set_filter_priority) = signal(Option::<TaskPriority>::None);

    let handle_input_title = move |ev: web_sys::Event| {
        let input = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        set_new_task_title.set(input.value());
    };

    let handle_input_description = move |ev: web_sys::Event| {
        let input = ev.target().unwrap().dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
        set_new_task_description.set(input.value());
    };

    let handle_priority_change = move |ev: web_sys::Event| {
        let select = ev.target().unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        let priority = match select.value().as_str() {
            "Low" => TaskPriority::Low,
            "Medium" => TaskPriority::Medium,
            "High" => TaskPriority::High,
            "Critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        };
        set_new_task_priority.set(priority);
    };

    let handle_add_task = move |_| {
        let title = new_task_title.get();
        let description = new_task_description.get();
        
        if !title.trim().is_empty() {
            let task = Task::new(title, description, new_task_priority.get());
            set_manager.update(|manager_val| {
                let _ = manager_val.add_task(task);
            });
            set_new_task_title.set(String::new());
            set_new_task_description.set(String::new());
        }
    };

    let handle_status_filter = move |ev: web_sys::Event| {
        let select = ev.target().unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        let status = match select.value().as_str() {
            "Todo" => Some(TaskStatus::Todo),
            "InProgress" => Some(TaskStatus::InProgress),
            "Done" => Some(TaskStatus::Done),
            "Cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        };
        set_filter_status.set(status);
    };

    let handle_priority_filter = move |ev: web_sys::Event| {
        let select = ev.target().unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        let priority = match select.value().as_str() {
            "Low" => Some(TaskPriority::Low),
            "Medium" => Some(TaskPriority::Medium),
            "High" => Some(TaskPriority::High),
            "Critical" => Some(TaskPriority::Critical),
            _ => None,
        };
        set_filter_priority.set(priority);
    };

    let handle_merge_dummy = move |_| {
        let other_user_id = Uuid::new_v4();
        let mut other_manager = TaskManager::new(other_user_id);
        
        let task1 = Task::new("Dummy Task 1".to_string(), "First dummy task".to_string(), TaskPriority::High);
        let task2 = Task::new("Dummy Task 2".to_string(), "Second dummy task".to_string(), TaskPriority::Low);
        let _ = other_manager.add_task(task1);
        let _ = other_manager.add_task(task2);
        
        set_manager.update(|manager_val| {
            let _ = manager_val.merge(&other_manager);
        });
    };

    let filtered_tasks = move || {
        let tasks = manager.get().get_tasks();
        let status_filter = filter_status.get();
        let priority_filter = filter_priority.get();
        
        tasks.into_iter()
            .filter(|task| {
                let status_match = status_filter.as_ref().map_or(true, |s| task.status == *s);
                let priority_match = priority_filter.as_ref().map_or(true, |p| task.priority == *p);
                status_match && priority_match
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="task-manager-demo">
            <h1>"Collaborative Task Manager Demo"</h1>
            <p>"This demo showcases a collaborative task manager using the LSEQ CRDT for ordered task management."</p>

            <div class="add-task-section">
                <h3>"Add New Task"</h3>
                <div class="task-form">
                    <input type="text"
                        prop:value=new_task_title
                        on:input=handle_input_title
                        placeholder="Task title..."
                    />
                    <textarea
                        prop:value=new_task_description
                        on:input=handle_input_description
                        placeholder="Task description..."
                        rows="3"
                    ></textarea>
                    <select on:change=handle_priority_change>
                        <option value="Low">"Low Priority"</option>
                        <option value="Medium" selected="true">"Medium Priority"</option>
                        <option value="High">"High Priority"</option>
                        <option value="Critical">"Critical Priority"</option>
                    </select>
                    <button on:click=handle_add_task>"Add Task"</button>
                </div>
            </div>

            <div class="filters-section">
                <h3>"Filters"</h3>
                <div class="filters">
                    <select on:change=handle_status_filter>
                        <option value="">"All Statuses"</option>
                        <option value="Todo">"Todo"</option>
                        <option value="InProgress">"In Progress"</option>
                        <option value="Done">"Done"</option>
                        <option value="Cancelled">"Cancelled"</option>
                    </select>
                    <select on:change=handle_priority_filter>
                        <option value="">"All Priorities"</option>
                        <option value="Low">"Low Priority"</option>
                        <option value="Medium">"Medium Priority"</option>
                        <option value="High">"High Priority"</option>
                        <option value="Critical">"Critical Priority"</option>
                    </select>
                    <button on:click=handle_merge_dummy>"Merge Dummy Tasks"</button>
                </div>
            </div>

            <div class="tasks-section">
                <h3>"Tasks (" {move || filtered_tasks().len()} ")"</h3>
                <div class="tasks-list">
                    {move || {
                        filtered_tasks().into_iter().map(|task| {
                            let task_priority_1 = task.priority.clone();
                            let task_priority_2 = task.priority.clone();
                            let task_status = task.status.clone();
                            view! {
                                <div class="task-item" class:high-priority=move || task_priority_1 == TaskPriority::High || task_priority_1 == TaskPriority::Critical>
                                    <div class="task-header">
                                        <h4>{task.title.clone()}</h4>
                                        <span class="priority" class:critical=move || task_priority_2 == TaskPriority::Critical>
                                            {task_priority_2.to_string()}
                                        </span>
                                    </div>
                                    <p class="task-description">{task.description.clone()}</p>
                                    <div class="task-meta">
                                        <span class="status" class:done=move || task_status == TaskStatus::Done>
                                            {task_status.to_string()}
                                        </span>
                                        <span class="created-at">
                                            "Created: " {task.created_at.format("%Y-%m-%d %H:%M").to_string()}
                                        </span>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>

            <div class="manager-info">
                <h3>"Manager Info:"</h3>
                <p>"Total Tasks: " {move || manager.get().len()}</p>
                <p>"Is Empty: " {move || manager.get().is_empty().to_string()}</p>
                <p>"User ID: " {move || manager.get().get_user_id().to_string()}</p>
            </div>

            <div class="instructions">
                <h3>"Instructions:"</h3>
                <ul>
                    <li>"Add new tasks using the form above."</li>
                    <li>"Filter tasks by status or priority using the dropdowns."</li>
                    <li>"Click 'Merge Dummy Tasks' to simulate collaboration with another user."</li>
                    <li>"Tasks are ordered using LSEQ CRDT for conflict-free collaboration."</li>
                </ul>
            </div>
        </div>
        <style>
            {r#"
            .task-manager-demo {
                font-family: Arial, sans-serif;
                max-width: 1000px;
                margin: 20px auto;
                padding: 20px;
                border: 1px solid #ccc;
                border-radius: 8px;
                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                background-color: #f9f9f9;
            }
            h1, h3 {
                color: #333;
            }
            .add-task-section, .filters-section, .tasks-section, .manager-info, .instructions {
                margin-bottom: 30px;
                padding: 15px;
                border: 1px solid #ddd;
                border-radius: 6px;
                background-color: white;
            }
            .task-form {
                display: flex;
                flex-direction: column;
                gap: 10px;
            }
            .task-form input, .task-form textarea, .task-form select {
                padding: 8px;
                border: 1px solid #ddd;
                border-radius: 4px;
                font-size: 14px;
            }
            .task-form button {
                padding: 10px 15px;
                background-color: #007bff;
                color: white;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-size: 14px;
            }
            .task-form button:hover {
                background-color: #0056b3;
            }
            .filters {
                display: flex;
                gap: 10px;
                align-items: center;
            }
            .filters select {
                padding: 8px;
                border: 1px solid #ddd;
                border-radius: 4px;
            }
            .filters button {
                padding: 8px 15px;
                background-color: #28a745;
                color: white;
                border: none;
                border-radius: 4px;
                cursor: pointer;
            }
            .filters button:hover {
                background-color: #218838;
            }
            .tasks-list {
                display: flex;
                flex-direction: column;
                gap: 15px;
            }
            .task-item {
                padding: 15px;
                border: 1px solid #ddd;
                border-radius: 6px;
                background-color: #f8f9fa;
                transition: all 0.2s;
            }
            .task-item.high-priority {
                border-left: 4px solid #ffc107;
                background-color: #fff3cd;
            }
            .task-item.high-priority.critical {
                border-left-color: #dc3545;
                background-color: #f8d7da;
            }
            .task-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 8px;
            }
            .task-header h4 {
                margin: 0;
                color: #333;
            }
            .priority {
                padding: 4px 8px;
                border-radius: 12px;
                font-size: 12px;
                font-weight: bold;
                background-color: #6c757d;
                color: white;
            }
            .priority.critical {
                background-color: #dc3545;
            }
            .task-description {
                margin: 8px 0;
                color: #666;
                line-height: 1.4;
            }
            .task-meta {
                display: flex;
                justify-content: space-between;
                align-items: center;
                font-size: 12px;
                color: #666;
            }
            .status {
                padding: 2px 6px;
                border-radius: 8px;
                background-color: #e9ecef;
                color: #495057;
            }
            .status.done {
                background-color: #d4edda;
                color: #155724;
            }
            .created-at {
                font-style: italic;
            }
            .manager-info p {
                margin: 5px 0;
                font-family: monospace;
                background-color: #e9ecef;
                padding: 5px;
                border-radius: 3px;
            }
            ul {
                list-style-type: disc;
                margin-left: 20px;
            }
            li {
                margin-bottom: 5px;
            }
            "#}
        </style>
    }
}

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <TaskManagerDemo/> })
}
