-- Clear all tables including queues
DELETE FROM app_usage_sessions;
DELETE FROM work_sessions;
DELETE FROM offline_queue;
DELETE FROM event_queue;
DELETE FROM heartbeat_queue;

-- Reset auto-increment counters
DELETE FROM sqlite_sequence WHERE name IN ('app_usage_sessions', 'work_sessions', 'offline_queue', 'event_queue', 'heartbeat_queue');

-- Show counts to verify
SELECT 'app_usage_sessions' as table_name, COUNT(*) as count FROM app_usage_sessions
UNION ALL
SELECT 'work_sessions', COUNT(*) FROM work_sessions
UNION ALL
SELECT 'offline_queue', COUNT(*) FROM offline_queue
UNION ALL
SELECT 'event_queue', COUNT(*) FROM event_queue
UNION ALL
SELECT 'heartbeat_queue', COUNT(*) FROM heartbeat_queue;



