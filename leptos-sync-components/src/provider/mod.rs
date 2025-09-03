use leptos::*;

#[component]
pub fn LocalFirstProvider(
    children: Children,
) -> impl IntoView {
    view! {
        <div>
            {children()}
        </div>
    }
}
