use rocket::tokio::sync::Mutex;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct WebCache {
    pub token: Option<Arc<String>>,
    pub queue: Arc<Mutex<BTreeMap<String, VecDeque<Vec<u8>>>>>,
}

impl WebCache {
    pub fn from_token(token: impl ToString) -> Self {
        WebCache {
            token: Some(Arc::new(token.to_string())),
            ..Default::default()
        }
    }
    pub fn new(cfg: &crate::config::Config) -> Self {
        if let Some(auth) = &cfg.auth {
           WebCache::from_token(&auth.token)
        } else {
            WebCache::default()
        }
    }
    /// 检查指定的消息队列中是否存在消息
    #[allow(unused)]
    pub async fn queue_exists_msg(&self, queue: &str) -> bool {
        self.queue
            .lock()
            .await
            .get(queue)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
    /// 检查指定的任务队列中的消息数量
    #[allow(unused)]
    pub async fn queue_len(&self, queue: &str) -> usize {
        self.queue
            .lock()
            .await
            .get(queue)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// 从指定的任务队列中推出一条消息
    pub async fn queue_pop_msg(&self, queue: &str) -> Option<Vec<u8>> {
        if let Some(q) = self.queue.lock().await.get_mut(queue) {
            q.pop_back()
        } else {
            None
        }
    }
    /// 从指定的任务队列中获取一条消息
    pub async fn queue_pick_msg(&self, queue: &str,index: usize) -> Option<Vec<u8>> {
        if let Some(q) = self.queue.lock().await.get_mut(queue) {
            return q.get(index).cloned();
        } else {
            None
        }
    }
    /// 向指定的任务队列推一条消息
    pub async fn queue_push_msg(&self, queue: &str, msg: Vec<u8>) {
        if let Some(q) = self.queue.lock().await.get_mut(queue) {
            q.push_front(msg)
        } else {
            self.queue.lock().await.insert(queue.to_string(), VecDeque::from([msg]));
        }
    }
}
