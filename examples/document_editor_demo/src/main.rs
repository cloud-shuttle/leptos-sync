use leptos::prelude::*;
use document_editor_demo::document_editor::{DocumentEditor, DocumentNode, NodeType};
use leptos_sync_core::crdt::Mergeable;
use uuid::Uuid;
use wasm_bindgen::JsCast;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    let (editor, set_editor) = signal(DocumentEditor::new(Uuid::new_v4()));
    let (section_title, set_section_title) = signal(String::new());
    let (paragraph_content, set_paragraph_content) = signal(String::new());
    let (heading_title, set_heading_title) = signal(String::new());
    let (heading_level, set_heading_level) = signal(1);

    let handle_add_section = move |_| {
        let title = section_title.get();
        if !title.is_empty() {
            set_editor.update(|editor| {
                let _ = editor.add_section(title.clone());
            });
            set_section_title.set(String::new());
        }
    };

    let handle_add_paragraph = move |_| {
        let content = paragraph_content.get();
        if !content.is_empty() {
            set_editor.update(|editor| {
                if let Some(section_pos) = editor.get_first_position() {
                    let _ = editor.add_paragraph(&section_pos, content.clone());
                }
            });
            set_paragraph_content.set(String::new());
        }
    };

    let handle_add_heading = move |_| {
        let title = heading_title.get();
        let level = heading_level.get();
        if !title.is_empty() {
            set_editor.update(|editor| {
                if let Some(section_pos) = editor.get_first_position() {
                    let _ = editor.add_heading(&section_pos, title.clone(), level);
                }
            });
            set_heading_title.set(String::new());
        }
    };

    let handle_merge_dummy = move |_| {
        set_editor.update(|editor| {
            let mut other_editor = DocumentEditor::new(Uuid::new_v4());
            let section_id = other_editor.add_section("Dummy Section".to_string()).unwrap();
            let _ = other_editor.add_paragraph(&section_id, "This is a merged paragraph.".to_string());
            let _ = editor.merge(&other_editor);
        });
    };

    let document_content = move || {
        let editor_val = editor.get();
        if !editor_val.is_empty() {
            let tree = editor_val.get_document_tree();
            format!("Document has {} nodes", tree.unwrap().children.len())
        } else {
            "No document content yet. Add a section to get started!".to_string()
        }
    };

    view! {
        <div class="document-editor">
            <h1>"Document Editor Demo - Yjs Tree CRDT"</h1>
            
            <div class="controls">
                <div class="control-group">
                    <h3>"Add Section"</h3>
                    <input
                        type="text"
                        placeholder="Section title"
                        prop:value=section_title
                        on:input=move |ev| {
                            let target = ev.target().unwrap();
                            let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                            set_section_title.set(input.value());
                        }
                    />
                    <button on:click=handle_add_section>"Add Section"</button>
                </div>

                <div class="control-group">
                    <h3>"Add Paragraph"</h3>
                    <input
                        type="text"
                        placeholder="Paragraph content"
                        prop:value=paragraph_content
                        on:input=move |ev| {
                            let target = ev.target().unwrap();
                            let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                            set_paragraph_content.set(input.value());
                        }
                    />
                    <button on:click=handle_add_paragraph>"Add Paragraph"</button>
                </div>

                <div class="control-group">
                    <h3>"Add Heading"</h3>
                    <input
                        type="text"
                        placeholder="Heading title"
                        prop:value=heading_title
                        on:input=move |ev| {
                            let target = ev.target().unwrap();
                            let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                            set_heading_title.set(input.value());
                        }
                    />
                    <select
                        prop:value=heading_level
                        on:change=move |ev| {
                            let target = ev.target().unwrap();
                            let select = target.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                            set_heading_level.set(select.value().parse().unwrap_or(1));
                        }
                    >
                        <option value="1">"H1"</option>
                        <option value="2">"H2"</option>
                        <option value="3">"H3"</option>
                        <option value="4">"H4"</option>
                        <option value="5">"H5"</option>
                        <option value="6">"H6"</option>
                    </select>
                    <button on:click=handle_add_heading>"Add Heading"</button>
                </div>

                <div class="control-group">
                    <button on:click=handle_merge_dummy>"Merge Demo Data"</button>
                </div>
            </div>

            <div class="document-view">
                <h2>"Document Structure"</h2>
                <div class="tree-view">
                    <p>{document_content}</p>
                </div>
            </div>
        </div>
    }
}