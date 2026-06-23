// ETF Tracker — Main Entry Point
// Windows 데스크톱 ETF 데이터 트래커

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    etf_tracker_lib::run()
}