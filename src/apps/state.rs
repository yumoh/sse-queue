use rocket::tokio::sync::Mutex;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::Arc;

type Locker<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone, Default)]
pub struct WebCache {
    pub token: Option<Arc<String>>,
    pub queue: Locker<BTreeMap<String, Locker<VecDeque<Vec<u8>>>>>,
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
        let queue = { self.queue.lock().await.get(queue).cloned() };
        if let Some(queue) = queue {
            !queue.lock().await.is_empty()
        } else {
            false
        }
    }
    /// 检查指定的任务队列中的消息数量
    #[allow(unused)]
    pub async fn queue_len(&self, queue: &str) -> usize {
        let queue = { self.queue.lock().await.get(queue).cloned() };
        if let Some(queue) = queue {
            queue.lock().await.len()
        } else {
            0
        }
    }

    /// 从指定的任务队列中推出一条消息
    pub async fn queue_pop_msg(&self, queue: &str) -> Option<Vec<u8>> {
        let queue = { self.queue.lock().await.get(queue).cloned() };
        if let Some(queue) = queue {
            queue.lock().await.pop_back()
        } else {
            None
        }
    }
    /// 从指定的任务队列中获取一条消息
    pub async fn queue_pick_msg(&self, queue: &str, index: usize) -> Option<Vec<u8>> {
        let queue = { self.queue.lock().await.get(queue).cloned() };
        if let Some(queue) = queue {
            queue.lock().await.get(index).cloned()
        } else {
            None
        }
    }
    pub async fn queue_last(&self,queue: &str) -> Option<Vec<u8>>{
        let queue = { self.queue.lock().await.get(queue).cloned() };
        if let Some(queue) = queue {
            queue.lock().await.front().cloned()
        } else {
            None
        }
    }
    pub async fn queue_first(&self,queue: &str) -> Option<Vec<u8>>{
        let queue = { self.queue.lock().await.get(queue).cloned() };
        if let Some(queue) = queue {
            queue.lock().await.back().cloned()
        } else {
            None
        }
    }
    /// 向指定的任务队列推一条消息
    pub async fn queue_push_msg(&self, queue_name: &str, msg: Vec<u8>) {
        let queue = { self.queue.lock().await.get(queue_name).cloned() };
        if let Some(queue) = queue {
            queue.lock().await.push_front(msg);
        } else {
            self.queue
                    .lock()
                    .await
                    .entry(queue_name.to_string())
                    .or_insert_with(|| Arc::new(Mutex::new(VecDeque::new()))).lock().await.push_front(msg);
        }
    }
}
