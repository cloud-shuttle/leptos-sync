use leptos::prelude::*;
use text_editor_demo::text_editor::TextEditor;
use uuid::Uuid;
use web_sys::KeyboardEvent;
use wasm_bindgen::JsCast;

/// Main component for the collaborative text editor demo
#[component]
pub fn TextEditorDemo() -> impl IntoView {
    let user_id = Uuid::new_v4();
    let (editor, set_editor) = signal(TextEditor::new(user_id));
    let (show_cursor_positions, set_show_cursor_positions) = signal(false);

    // Handle text input
    let handle_input = move |ev: web_sys::Event| {
        let input = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let value = input.value();
        
        // Clear the input
        input.set_value("");
        
        // Insert each character
        for ch in value.chars() {
            set_editor.update(|editor| {
                editor.insert_char(ch).unwrap();
            });
        }
    };

    // Handle keyboard shortcuts
    let handle_keydown = move |ev: KeyboardEvent| {
        match ev.key().as_str() {
            "Backspace" => {
                ev.prevent_default();
                set_editor.update(|editor| {
                    editor.delete_char().unwrap();
                });
            }
            "Enter" => {
                ev.prevent_default();
                set_editor.update(|editor| {
                    editor.insert_char('\n').unwrap();
                });
            }
            "z" if ev.ctrl_key() || ev.meta_key() => {
                ev.prevent_default();
                set_editor.update(|editor| {
                    editor.undo().unwrap();
                });
            }
            "y" if ev.ctrl_key() || ev.meta_key() => {
                ev.prevent_default();
                set_editor.update(|editor| {
                    editor.redo().unwrap();
                });
            }
            _ => {}
        }
    };

    // Simulate collaborative editing
    let simulate_collaboration = move |_| {
        let mut simulated_editor = TextEditor::new(Uuid::new_v4());
        simulated_editor.insert_char('S').unwrap();
        simulated_editor.insert_char('i').unwrap();
        simulated_editor.insert_char('m').unwrap();
        simulated_editor.insert_char('u').unwrap();
        simulated_editor.insert_char('l').unwrap();
        simulated_editor.insert_char('a').unwrap();
        simulated_editor.insert_char('t').unwrap();
        simulated_editor.insert_char('e').unwrap();
        simulated_editor.insert_char('d').unwrap();
        simulated_editor.insert_char(' ').unwrap();
        simulated_editor.insert_char('t').unwrap();
        simulated_editor.insert_char('e').unwrap();
        simulated_editor.insert_char('x').unwrap();
        simulated_editor.insert_char('t').unwrap();
        
        set_editor.update(|editor| {
            editor.merge(&simulated_editor).unwrap();
        });
    };

    // Clear the editor
    let clear_editor = move |_| {
        set_editor.update(|editor| {
            *editor = TextEditor::new(user_id);
        });
    };

    view! {
        <div class="text-editor-demo">
            <h1>"Collaborative Text Editor Demo"</h1>
            <p>"This demo showcases a collaborative text editor using RGA (Replicated Growable Array) CRDT."</p>
            
            <div class="editor-controls">
                <div class="input-section">
                    <label for="text-input">"Type text:"</label>
                    <input
                        id="text-input"
                        type="text"
                        placeholder="Type here..."
                        on:input=handle_input
                        on:keydown=handle_keydown
                    />
                </div>
                
                <div class="button-section">
                    <button on:click=simulate_collaboration>
                        "Simulate Collaboration"
                    </button>
                    <button on:click=clear_editor>
                        "Clear Editor"
                    </button>
                    <button on:click=move |_| set_show_cursor_positions.update(|v| *v = !*v)>
                        {move || if show_cursor_positions.get() { "Hide" } else { "Show" } } " Cursor Positions"
                    </button>
                </div>
                
                <div class="undo-redo-section">
                    <button 
                        disabled=move || !editor.get().can_undo()
                        on:click=move |_| {
                            set_editor.update(|editor| {
                                editor.undo().unwrap();
                            });
                        }
                    >
                        "Undo"
                    </button>
                    <button 
                        disabled=move || !editor.get().can_redo()
                        on:click=move |_| {
                            set_editor.update(|editor| {
                                editor.redo().unwrap();
                            });
                        }
                    >
                        "Redo"
                    </button>
                </div>
            </div>
            
            <div class="editor-content">
                <div class="editor-display">
                    <h3>"Editor Content:"</h3>
                    <div class="text-content">
                        {move || editor.get().get_text()}
                    </div>
                </div>
                
                <div class="editor-stats">
                    <h3>"Editor Statistics:"</h3>
                    <div class="stats">
                        <p>"Length: " {move || editor.get().len()}</p>
                        <p>"Empty: " {move || editor.get().is_empty()}</p>
                        <p>"Can Undo: " {move || editor.get().can_undo()}</p>
                        <p>"Can Redo: " {move || editor.get().can_redo()}</p>
                        <p>"User ID: " {move || editor.get().get_user_id().to_string()}</p>
                    </div>
                </div>
            </div>
            
            <div class="instructions">
                <h3>"Instructions:"</h3>
                <ul>
                    <li>"Type in the input field to add text to the editor"</li>
                    <li>"Use Backspace to delete characters"</li>
                    <li>"Use Ctrl+Z (Cmd+Z on Mac) to undo"</li>
                    <li>"Use Ctrl+Y (Cmd+Y on Mac) to redo"</li>
                    <li>"Click 'Simulate Collaboration' to see how multiple users' changes merge"</li>
                </ul>
            </div>
        </div>
    }
}

/// Main function to run the demo
fn main() {
    console_error_panic_hook::set_once();
    
    mount_to_body(|| {
        view! {
            <div class="app">
                <TextEditorDemo />
                <TextEditorStyles />
            </div>
        }
    });
}

/// CSS styles for the text editor demo
#[component]
pub fn TextEditorStyles() -> impl IntoView {
    view! {
        <style>
            {r#"
            .text-editor-demo {
                max-width: 1200px;
                margin: 0 auto;
                padding: 20px;
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            }
            
            .text-editor-demo h1 {
                color: #333;
                text-align: center;
                margin-bottom: 30px;
            }
            
            .editor-controls {
                background: #f5f5f5;
                padding: 20px;
                border-radius: 8px;
                margin-bottom: 20px;
            }
            
            .input-section {
                margin-bottom: 15px;
            }
            
            .input-section label {
                display: block;
                margin-bottom: 5px;
                font-weight: bold;
            }
            
            .input-section input {
                width: 100%;
                padding: 10px;
                border: 1px solid #ddd;
                border-radius: 4px;
                font-size: 16px;
            }
            
            .button-section, .undo-redo-section {
                display: flex;
                gap: 10px;
                margin-bottom: 15px;
                flex-wrap: wrap;
            }
            
            .button-section button, .undo-redo-section button {
                padding: 8px 16px;
                border: none;
                border-radius: 4px;
                background: #007bff;
                color: white;
                cursor: pointer;
                font-size: 14px;
            }
            
            .button-section button:hover, .undo-redo-section button:hover {
                background: #0056b3;
            }
            
            .button-section button:disabled, .undo-redo-section button:disabled {
                background: #6c757d;
                cursor: not-allowed;
            }
            
            .editor-content {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 20px;
                margin-bottom: 20px;
            }
            
            .editor-display, .editor-stats {
                background: white;
                border: 1px solid #ddd;
                border-radius: 8px;
                padding: 20px;
            }
            
            .editor-display h3, .editor-stats h3 {
                margin-top: 0;
                color: #333;
            }
            
            .text-content {
                background: #f8f9fa;
                border: 1px solid #e9ecef;
                border-radius: 4px;
                padding: 15px;
                min-height: 100px;
                white-space: pre-wrap;
                font-family: 'Courier New', monospace;
                font-size: 14px;
                line-height: 1.5;
            }
            
            .stats p {
                margin: 5px 0;
                padding: 5px;
                background: #f8f9fa;
                border-radius: 4px;
            }
            
            .instructions {
                background: #fff3cd;
                border: 1px solid #ffeaa7;
                border-radius: 8px;
                padding: 20px;
            }
            
            .instructions h3 {
                margin-top: 0;
                color: #856404;
            }
            
            .instructions ul {
                margin: 0;
                padding-left: 20px;
            }
            
            .instructions li {
                margin-bottom: 5px;
                color: #856404;
            }
            
            @media (max-width: 768px) {
                .editor-content {
                    grid-template-columns: 1fr;
                }
                
                .button-section, .undo-redo-section {
                    flex-direction: column;
                }
            }
            "#
            }
        </style>
    }
}