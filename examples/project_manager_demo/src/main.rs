use leptos::prelude::*;
use project_manager_demo::project_manager::{
    ProjectManager, Task, TaskStatus, TaskPriority
};
use leptos_sync_core::crdt::Mergeable;
use uuid::Uuid;
use wasm_bindgen::JsCast;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let (manager, set_manager) = signal(ProjectManager::new(Uuid::new_v4()));
    let (task_title, set_task_title) = signal(String::new());
    let (task_description, set_task_description) = signal(String::new());
    let (task_priority, set_task_priority) = signal(TaskPriority::Medium);
    let (task_assignee, set_task_assignee) = signal(String::new());
    let (task_estimated_hours, set_task_estimated_hours) = signal(String::new());

    let handle_add_task = move |_| {
        let title = task_title.get();
        let description = task_description.get();
        if !title.is_empty() && !description.is_empty() {
            set_manager.update(|manager| {
                let mut task = Task::new(title.clone(), description.clone())
                    .with_priority(task_priority.get());
                
                if !task_assignee.get().is_empty() {
                    task = task.with_assignee(task_assignee.get());
                }
                
                if let Ok(hours) = task_estimated_hours.get().parse::<u32>() {
                    task = task.with_estimated_hours(hours);
                }
                
                let _ = manager.add_task(task);
            });
            
            set_task_title.set(String::new());
            set_task_description.set(String::new());
            set_task_assignee.set(String::new());
            set_task_estimated_hours.set(String::new());
        }
    };

    let handle_merge_demo = move |_| {
        set_manager.update(|manager| {
            let mut demo_manager = ProjectManager::new(Uuid::new_v4());
            
            // Create demo tasks
            let setup_task = Task::new("Project Setup".to_string(), "Initialize project structure".to_string())
                .with_priority(TaskPriority::High)
                .with_assignee("Alice".to_string())
                .with_estimated_hours(8);
            
            let dev_task = Task::new("Core Development".to_string(), "Implement main features".to_string())
                .with_priority(TaskPriority::Critical)
                .with_assignee("Bob".to_string())
                .with_estimated_hours(40);
            
            let test_task = Task::new("Testing".to_string(), "Test all functionality".to_string())
                .with_priority(TaskPriority::High)
                .with_assignee("Charlie".to_string())
                .with_estimated_hours(16);
            
            let _setup_id = demo_manager.add_task(setup_task).unwrap();
            let _dev_id = demo_manager.add_task(dev_task).unwrap();
            let _test_id = demo_manager.add_task(test_task).unwrap();
            
            let _ = manager.merge(&demo_manager);
        });
    };

    let tasks = move || {
        let manager_ref = manager.get();
        manager_ref.get_tasks().into_iter().map(|task| task.clone()).collect::<Vec<_>>()
    };
    let ready_tasks = move || {
        let manager_ref = manager.get();
        manager_ref.get_ready_tasks().into_iter().map(|task| task.clone()).collect::<Vec<_>>()
    };

    view! {
        <div class="project-manager">
            <h1>"Project Manager Demo - DAG CRDT"</h1>
            
            <div class="controls">
                <div class="control-group">
                    <h3>"Add New Task"</h3>
                    <input
                        type="text"
                        placeholder="Task title"
                        prop:value=task_title
                        on:input=move |ev| {
                            let target = ev.target().unwrap();
                            let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                            set_task_title.set(input.value());
                        }
                    />
                    <input
                        type="text"
                        placeholder="Task description"
                        prop:value=task_description
                        on:input=move |ev| {
                            let target = ev.target().unwrap();
                            let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                            set_task_description.set(input.value());
                        }
                    />
                    <select
                        prop:value=move || format!("{:?}", task_priority.get())
                        on:change=move |ev| {
                            let target = ev.target().unwrap();
                            let select = target.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                            let priority = match select.value().as_str() {
                                "Low" => TaskPriority::Low,
                                "Medium" => TaskPriority::Medium,
                                "High" => TaskPriority::High,
                                "Critical" => TaskPriority::Critical,
                                _ => TaskPriority::Medium,
                            };
                            set_task_priority.set(priority);
                        }
                    >
                        <option value="Low">"Low Priority"</option>
                        <option value="Medium">"Medium Priority"</option>
                        <option value="High">"High Priority"</option>
                        <option value="Critical">"Critical Priority"</option>
                    </select>
                    <input
                        type="text"
                        placeholder="Assignee (optional)"
                        prop:value=task_assignee
                        on:input=move |ev| {
                            let target = ev.target().unwrap();
                            let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                            set_task_assignee.set(input.value());
                        }
                    />
                    <input
                        type="number"
                        placeholder="Estimated hours (optional)"
                        prop:value=task_estimated_hours
                        on:input=move |ev| {
                            let target = ev.target().unwrap();
                            let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                            set_task_estimated_hours.set(input.value());
                        }
                    />
                    <button on:click=handle_add_task>"Add Task"</button>
                </div>

                <div class="control-group">
                    <button on:click=handle_merge_demo>"Load Demo Project"</button>
                </div>
            </div>

            <div class="project-view">
                <h2>"Project Overview"</h2>
                <p>{move || format!("Total tasks: {}", manager.get().len())}</p>
                <p>{move || format!("Ready to start: {}", ready_tasks().len())}</p>
                
                <h3>"All Tasks"</h3>
                <div class="task-list">
                    {move || tasks().into_iter().map(|task| {
                        let status_class = match task.status {
                            TaskStatus::NotStarted => "not-started",
                            TaskStatus::InProgress => "in-progress",
                            TaskStatus::Completed => "completed",
                            TaskStatus::Blocked => "blocked",
                        };
                        view! {
                            <div class=format!("task-node {}", status_class)>
                                <div class="task-title">{task.title.clone()}</div>
                                <div class="task-description">{task.description.clone()}</div>
                                <div class="task-meta">
                                    "Priority: " {format!("{:?}", task.priority)} " | "
                                    "Status: " {format!("{:?}", task.status)} " | "
                                    "Assignee: " {task.assignee.clone().unwrap_or_else(|| "Unassigned".to_string())} " | "
                                    "Hours: " {task.estimated_hours.map(|h| h.to_string()).unwrap_or_else(|| "N/A".to_string())}
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}