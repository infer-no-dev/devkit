#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn retry_policy_exponential_backoff_caps() {
        let policy = RetryPolicy {
            max_retries: 5,
            strategy: BackoffStrategy::Exponential { base_secs: 1, factor: 2, max_secs: 5 },
        };
        let d0 = policy.next_delay(0).unwrap().as_secs();
        let d1 = policy.next_delay(1).unwrap().as_secs();
        let d2 = policy.next_delay(2).unwrap().as_secs();
        let d3 = policy.next_delay(3).unwrap().as_secs();
        let d4 = policy.next_delay(4).unwrap().as_secs();
        let none = policy.next_delay(5);
        assert_eq!((d0, d1, d2, d3, d4), (1, 2, 4, 5, 5));
        assert!(none.is_none());
    }

    #[tokio::test]
    async fn snapshot_store_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = FileTaskSnapshotStore::new(dir.path()).await.unwrap();
        let task = AgentTask::new("test".into(), "desc".into(), serde_json::Value::Null);
        let mut snap = TaskSnapshot::new_pending(task.clone());
        store.save(&snap).await.unwrap();
        let loaded = store.load(&task.id).await.unwrap().unwrap();
        assert_eq!(loaded.task_id, task.id);
        assert_eq!(loaded.status, TaskSnapshotStatus::Pending);

        // Update and list
        snap.status = TaskSnapshotStatus::Failed;
        snap.last_error = Some("boom".into());
        store.save(&snap).await.unwrap();
        let all = store.list().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].status, TaskSnapshotStatus::Failed);

        // Delete
        store.delete(&task.id).await.unwrap();
        let none = store.load(&task.id).await.unwrap();
        assert!(none.is_none());
    }
}
