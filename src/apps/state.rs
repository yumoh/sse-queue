use rocket::tokio::sync::Mutex;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
use rocket::tokio::fs::{self,File};
type Locker<T> = Arc<Mutex<T>>;

type SingleFile = Locker<(AtomicUsize,Option<File>)>;

#[derive(Debug, Clone, Default)]
pub struct WebCache {
    pub token: Option<Arc<String>>,
    pub queue: Locker<BTreeMap<String, Locker<VecDeque<Vec<u8>>>>>,
    pub data_workspace: Arc<std::path::PathBuf>,
    pub cache_logs: Locker<BTreeMap<String,Locker<File>>>,
    pub cache_append_fs: Locker<BTreeMap<String,SingleFile>>,
}

impl WebCache {
    // pub fn from_token(token: impl ToString,storage_dir: &std::path::PathBuf) -> Self {
    //     WebCache {
    //         token: Some(Arc::new(token.to_string())),
    //         data_workspace: Arc::new(storage_dir.clone()),
    //         ..Default::default()
    //     }
    // }
    pub fn new(cfg: &crate::config::Config) -> std::io::Result<Self> {
        let slf = if let Some(auth) = &cfg.auth {
            let token = &auth.token;
            let storage_dir = cfg.data_workspace()?;
            WebCache {
                token: Some(Arc::new(token.to_string())),
                data_workspace: Arc::new(storage_dir),
                ..Default::default()
            }
        } else {
            let storage_dir = cfg.data_workspace()?;
            WebCache {
                data_workspace: Arc::new(storage_dir),
                ..Default::default()
            }
        };
        Ok(slf)
    }
    /// 存储空间内打开一个子目录
    pub fn open_data_dir(&self,bucket: &str) -> std::path::PathBuf {
        // log::debug!("data workspace: {}",self.data_workspace.display());
        let mut workspace = std::path::PathBuf::from(self.data_workspace.as_ref());
        workspace.push(bucket);
        workspace
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
    pub async fn queue_listen(&self,queue_name:&str) -> Locker<VecDeque<Vec<u8>>> {
        let queue = { self.queue.lock().await.get(queue_name).cloned() };
        if let Some(queue) = queue {
            queue
        } else {
            let  queue=Locker::new(Default::default());
            self.queue.lock().await.insert(queue_name.to_string(), queue.clone());
            queue
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
    pub async fn open_online_log(&self,channel:&str,name:&str) -> std::io::Result<Locker<File>> {
        let cache_key = format!("{channel}/{name}");
        let log = self.cache_logs.lock().await.get(&cache_key).cloned();
        if let Some(log) = log {
            Ok(log)
        } else {
            let data_dir = self.open_data_dir(channel);
            if !fs::try_exists(&data_dir).await.unwrap_or(false) {
                fs::create_dir_all(&data_dir).await?
            }
            let path = data_dir.join(name);
            let file = fs::OpenOptions::new().append(true).create(true).open(&path).await?;
            let arc_file = Arc::new(Mutex::new(file));
            let arc_file = self.cache_logs.lock().await.entry(cache_key).or_insert(arc_file).clone();
            Ok(arc_file)
        }
    }
    pub async fn close_online_log(&self,channel:&str,name:&str) {
        let cache_key = format!("{channel}/{name}");
        self.cache_logs.lock().await.remove(&cache_key);
    }
    pub async fn open_append_file(&self,bucket:&str,name:&str) -> std::io::Result<SingleFile> {
        let cache_key = format!("{bucket}/{name}");
        let single_file = self.cache_append_fs.lock().await.entry(cache_key).or_insert_with(|| {Default::default()}).clone();
        {
            let mut lockf = single_file.lock().await;
            if lockf.1.is_none() {
                let data_dir = self.open_data_dir(bucket);
                if !fs::try_exists(&data_dir).await.unwrap_or(false) {
                    fs::create_dir_all(&data_dir).await?
                }
                let path = data_dir.join(name);
                // let file = File::create(&path).await?;
                let file = fs::OpenOptions::new().append(true).create(true).open(&path).await?;
                lockf.1= Some(file);
            } 
            lockf.0.fetch_add(1, Ordering::Relaxed);
        }
        Ok(single_file)
    }
    
    pub async fn close_append_file(&self,bucket:&str,name:&str) -> std::io::Result<()> {
        let cache_key = format!("{bucket}/{name}");
        self.cache_append_fs.lock().await.remove(&cache_key);
        Ok(())
    }
}
