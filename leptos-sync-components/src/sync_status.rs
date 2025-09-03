//! Enhanced sync status component for real-time synchronization

use leptos::prelude::*;
use leptos_sync_core::{
    sync::{SyncState, PeerInfo, PeerSyncStatus},
    transport::SyncTransport,
};
use serde::{Deserialize, Serialize};

/// Props for the sync status component
#[derive(Props, Clone)]
pub struct SyncStatusProps {
    pub sync_state: Signal<SyncState>,
    pub peer_count: Signal<usize>,
    pub is_online: Signal<bool>,
    pub on_start_sync: Callback<()>,
    pub on_stop_sync: Callback<()>,
    pub on_connect: Callback<()>,
    pub on_disconnect: Callback<()>,
}

/// Enhanced sync status component
#[component]
pub fn SyncStatusIndicator(props: SyncStatusProps) -> impl IntoView {
    let sync_state = props.sync_state;
    let peer_count = props.peer_count;
    let is_online = props.is_online;

    let status_color = move || {
        match sync_state.get() {
            SyncState::NotSynced => "text-gray-500",
            SyncState::Syncing => "text-blue-500",
            SyncState::Synced => "text-green-500",
            SyncState::Failed(_) => "text-red-500",
            SyncState::ResolvingConflicts => "text-yellow-500",
            SyncState::Offline => "text-gray-400",
        }
    };

    let status_icon = move || {
        match sync_state.get() {
            SyncState::NotSynced => "⏸️",
            SyncState::Syncing => "🔄",
            SyncState::Synced => "✅",
            SyncState::Failed(_) => "❌",
            SyncState::ResolvingConflicts => "⚠️",
            SyncState::Offline => "📡",
        }
    };

    let status_text = move || {
        match sync_state.get() {
            SyncState::NotSynced => "Not Syncing",
            SyncState::Syncing => "Synchronizing...",
            SyncState::Synced => "Synchronized",
            SyncState::Failed(e) => format!("Failed: {}", e),
            SyncState::ResolvingConflicts => "Resolving Conflicts",
            SyncState::Offline => "Offline",
        }
    };

    let connection_status = move || {
        if is_online.get() {
            "🟢 Online"
        } else {
            "🔴 Offline"
        }
    };

    view! {
        <div class="sync-status-indicator">
            <div class="status-header">
                <h3 class="text-lg font-semibold mb-2">"Synchronization Status"</h3>
            </div>
            
            <div class="status-grid">
                <div class="status-item">
                    <span class="status-icon">{status_icon}</span>
                    <div class="status-details">
                        <span class="status-text {status_color}">{status_text}</span>
                        <span class="status-label">"Sync State"</span>
                    </div>
                </div>
                
                <div class="status-item">
                    <span class="status-icon">"👥"</span>
                    <div class="status-details">
                        <span class="status-text">{peer_count}</span>
                        <span class="status-label">"Connected Peers"</span>
                    </div>
                </div>
                
                <div class="status-item">
                    <span class="status-icon">"🌐"</span>
                    <div class="status-details">
                        <span class="status-text">{connection_status}</span>
                        <span class="status-label">"Connection"</span>
                    </div>
                </div>
            </div>
            
            <div class="sync-controls">
                {move || {
                    match sync_state.get() {
                        SyncState::NotSynced | SyncState::Failed(_) | SyncState::Offline => {
                            view! {
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| props.on_start_sync.call(())
                                >
                                    "Start Sync"
                                </button>
                            }
                        }
                        SyncState::Syncing | SyncState::ResolvingConflicts => {
                            view! {
                                <button
                                    class="btn btn-secondary"
                                    on:click=move |_| props.on_stop_sync.call(())
                                >
                                    "Stop Sync"
                                </button>
                            }
                        }
                        SyncState::Synced => {
                            view! {
                                <div class="flex gap-2">
                                    <button
                                        class="btn btn-secondary"
                                        on:click=move |_| props.on_stop_sync.call(())
                                    >
                                        "Stop Sync"
                                    </button>
                                    <button
                                        class="btn btn-outline"
                                        on:click=move |_| props.on_start_sync.call(())
                                    >
                                        "Resync"
                                    </button>
                                </div>
                            }
                        }
                    }
                }}
                
                <div class="connection-controls">
                    {move || {
                        if is_online.get() {
                            view! {
                                <button
                                    class="btn btn-outline btn-sm"
                                    on:click=move |_| props.on_disconnect.call(())
                                >
                                    "Disconnect"
                                </button>
                            }
                        } else {
                            view! {
                                <button
                                    class="btn btn-outline btn-sm"
                                    on:click=move |_| props.on_connect.call(())
                                >
                                    "Connect"
                                </button>
                            }
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// Props for the peer list component
#[derive(Props, Clone)]
pub struct PeerListProps {
    pub peers: Signal<Vec<(String, PeerInfo)>>,
}

/// Peer list component
#[component]
pub fn PeerList(props: PeerListProps) -> impl IntoView {
    let peers = props.peers;

    view! {
        <div class="peer-list">
            <h4 class="text-md font-semibold mb-2">"Connected Peers"</h4>
            {move || {
                let peer_list = peers.get();
                if peer_list.is_empty() {
                    view! {
                        <p class="text-gray-500 text-sm">"No peers connected"</p>
                    }
                } else {
                    view! {
                        <div class="peer-grid">
                            {peer_list.into_iter().map(|(id, peer)| {
                                let peer_status = move || {
                                    match peer.sync_status {
                                        PeerSyncStatus::Never => "Never Synced",
                                        PeerSyncStatus::Success { timestamp } => {
                                            format!("Last Sync: {}", timestamp.format("%H:%M:%S"))
                                        }
                                        PeerSyncStatus::Failed { timestamp, error } => {
                                            format!("Failed: {} ({})", timestamp.format("%H:%M:%S"), error)
                                        }
                                        PeerSyncStatus::Syncing { started } => {
                                            format!("Syncing since {}", started.format("%H:%M:%S"))
                                        }
                                    }
                                };
                                
                                let status_color = move || {
                                    match peer.sync_status {
                                        PeerSyncStatus::Never => "text-gray-500",
                                        PeerSyncStatus::Success { .. } => "text-green-500",
                                        PeerSyncStatus::Failed { .. } => "text-red-500",
                                        PeerSyncStatus::Syncing { .. } => "text-blue-500",
                                    }
                                };
                                
                                view! {
                                    <div class="peer-item">
                                        <div class="peer-header">
                                            <span class="peer-id">{id}</span>
                                            <span class="peer-status {status_color}">{peer_status}</span>
                                        </div>
                                        <div class="peer-details">
                                            <span class="peer-last-seen">
                                                "Last seen: {peer.last_seen.format("%H:%M:%S")}"
                                            </span>
                                            <span class="peer-online">
                                                if peer.is_online { "🟢 Online" } else { "🔴 Offline" }
                                            </span>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }
            }}
        </div>
    }
}

/// Props for the conflict resolver component
#[derive(Props, Clone)]
pub struct ConflictResolverProps<T> {
    pub local_value: Signal<T>,
    pub remote_value: Signal<T>,
    pub on_resolve: Callback<T>,
    pub on_keep_local: Callback<()>,
    pub on_keep_remote: Callback<()>,
}

/// Conflict resolver component
#[component]
pub fn ConflictResolver<T: Clone + 'static>(props: ConflictResolverProps<T>) -> impl IntoView {
    let local_value = props.local_value;
    let remote_value = props.remote_value;

    view! {
        <div class="conflict-resolver">
            <h4 class="text-md font-semibold mb-3">"Conflict Resolution Required"</h4>
            <p class="text-sm text-gray-600 mb-4">
                "A conflict was detected between your local data and data from another peer. Choose how to resolve it:"
            </p>
            
            <div class="conflict-options">
                <div class="option-group">
                    <h5 class="text-sm font-medium mb-2">"Your Local Data:"</h5>
                    <div class="value-display">
                        <pre class="text-xs bg-gray-100 p-2 rounded">
                            {move || format!("{:#?}", local_value.get())}
                        </pre>
                    </div>
                    <button
                        class="btn btn-outline btn-sm mt-2"
                        on:click=move |_| props.on_keep_local.call(())
                    >
                        "Keep Local"
                    </button>
                </div>
                
                <div class="option-group">
                    <h5 class="text-sm font-medium mb-2">"Remote Data:"</h5>
                    <div class="value-display">
                        <pre class="text-xs bg-gray-100 p-2 rounded">
                            {move || format!("{:#?}", remote_value.get())}
                        </pre>
                    </div>
                    <button
                        class="btn btn-outline btn-sm mt-2"
                        on:click=move |_| props.on_keep_remote.call(())
                    >
                        "Keep Remote"
                    </button>
                </div>
            </div>
            
            <div class="resolution-actions">
                <button
                    class="btn btn-primary"
                    on:click=move |_| {
                        // Merge both values
                        let local = local_value.get();
                        let remote = remote_value.get();
                        // In a real implementation, you'd merge them here
                        props.on_resolve.call(local)
                    }
                >
                    "Merge Both"
                </button>
            </div>
        </div>
    }
}
