// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod consent;
mod sampling;
mod screenshots;
mod storage;
mod api;
mod policy;
mod utils;
mod permissions;

use std::sync::Arc;
use tauri::{Manager, WindowEvent};
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton};
use tokio::sync::Mutex;
use utils::logging;

use crate::commands::*;
use crate::storage::AppState;

fn main() {
    // Initialize logging
    logging::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Arc::new(Mutex::new(AppState::new())))
        .invoke_handler(tauri::generate_handler![
            login,
            logout,
            get_auth_status,
            accept_consent,
            get_consent_status,
            clock_in,
            clock_out,
            get_work_session,
            get_recent_sessions,
            clear_local_database,
            trigger_sync,

            get_tracking_status,
            take_screenshot,
            get_current_app,
            send_diagnostics,
            get_permissions_status,
            request_permissions,
            get_app_info,
            send_app_focus_event,
            send_heartbeat,
            check_pending_jobs,
            get_idle_time,
            start_background_services,
            stop_background_services,
            pause_background_services,
            resume_background_services,
            get_background_service_state,
            get_app_usage_summary,
            get_usage_totals,
            get_current_app_session,
            get_detailed_idle_info,
            generate_today_report,
            generate_weekly_report,
            generate_monthly_summary,
            sync_app_rules,
            get_app_rules,
            get_rule_statistics
        ])
        .setup(|app| {
            // Set the global app state
            let app_state = app.state::<Arc<Mutex<AppState>>>();
            crate::storage::set_global_app_state(app_state.inner().clone());
            
            // Initialize the database directly
            let app_handle_for_bg = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::storage::database::init().await {
                    log::error!("Failed to initialize database: {}", e);
                } else {
                }
                
                if let Err(e) = crate::storage::app_usage::init_database().await {
                    log::error!("Failed to initialize app usage database: {}", e);
                } else {
                }
                
                if let Err(e) = crate::api::app_rules::initialize_app_rules().await {
                    log::error!("Failed to initialize app rules: {}", e);
                } else {
                }
                
                // Initialize power state monitoring
                crate::sampling::power_state::init();
                
                // Start background services
                crate::sampling::start_services().await;
                tokio::spawn(crate::sampling::start_queue_processing_service());
                
                // Start sync service for offline/online data synchronization
                tokio::spawn(crate::sampling::start_sync_service());
                
                // Start all sampling services - but only if user is authenticated AND clocked in
                // This prevents race conditions where services try to access empty global state
                tokio::spawn(async move {
                    // Wait for initial authentication check before starting services
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    
                    // Check if user is already authenticated AND has an active work session
                    if crate::sampling::is_authenticated().await && crate::sampling::is_clocked_in().await {
                        log::info!("User is authenticated and clocked in, starting background services");
                        crate::sampling::start_all_background_services(app_handle_for_bg).await;
                    } else {
                        log::info!("User is not authenticated or not clocked in, services will start after clock-in");
                    }
                    // If not authenticated or not clocked in, services will be started after clock-in
                });
            });
            
            // Create system tray
            let quit_i = MenuItem::with_id(app, "quit", "Quit TrackEx", true, None::<&str>)?;
            let pause_i = MenuItem::with_id(app, "pause", "Pause Tracking", true, None::<&str>)?;
            let resume_i = MenuItem::with_id(app, "resume", "Resume Tracking", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show TrackEx", true, None::<&str>)?;
            let diagnostics_i = MenuItem::with_id(app, "diagnostics", "Send Diagnostics", true, None::<&str>)?;
            
            let menu = MenuBuilder::new(app)
                .item(&show_i)
                .separator()
                .item(&pause_i)
                .item(&resume_i)
                .separator()
                .item(&diagnostics_i)
                .separator()
                .item(&quit_i)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("TrackEx Agent")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "pause" => {
                        println!("Pause tracking requested from tray");
                        // TODO: Implement pause logic
                    }
                    "resume" => {
                        println!("Resume tracking requested from tray");
                        // TODO: Implement resume logic
                    }
                    "diagnostics" => {
                        println!("Diagnostics requested from tray");
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        if let Some(app) = tray.app_handle().get_webview_window("main") {
                            let _ = app.show();
                            let _ = app.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Show main window on startup
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.center();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of closing
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}