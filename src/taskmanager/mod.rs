use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Mutex};
use std::thread;

pub type TaskId = usize;

pub struct Task {
    id: TaskId,
    task: Box<dyn FnOnce() + Send>,
    priority: i32,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.priority.partial_cmp(&self.priority)
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
    }
}

pub struct TaskManager {
    next_task_id: TaskId,
    task_queue: Arc<Mutex<BinaryHeap<Task>>>,
    task_dependencies: Arc<Mutex<HashMap<TaskId, Vec<TaskId>>>>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl TaskManager {
    pub fn new(num_threads: usize) -> Self {
        let task_queue = Arc::new(Mutex::new(BinaryHeap::<Task>::new()));
        let task_dependencies = Arc::new(Mutex::new(HashMap::<TaskId, Vec<TaskId>>::new()));
        let mut threads = Vec::new();

        for _ in 0..num_threads {
            let task_queue = task_queue.clone();
            let task_dependencies = task_dependencies.clone();
            let thread = thread::spawn(move || loop {
                let task = {
                    let mut queue = task_queue.lock().unwrap();
                    if queue.is_empty() {
                        None
                    } else if let Some(task) = queue.peek() {
                        let dependencies = task_dependencies.lock().unwrap();
                        if dependencies.contains_key(&task.id) {
                            None
                        } else {
                            Some(queue.pop().unwrap())
                        }
                    } else {
                        None
                    }
                };
                if let Some(task) = task {
                    (task.task)();
                    let mut dependencies = task_dependencies.lock().unwrap();
                    dependencies.retain(|_, v| {
                        v.retain(|&t| t != task.id);
                        !v.is_empty()
                    });
                } else {
                    thread::yield_now();
                }
            });
            threads.push(thread);
        }

        Self {
            next_task_id: 0,
            task_queue,
            task_dependencies,
            threads,
        }
    }

    pub fn add_task<T: FnOnce() + Send + 'static>(
        &mut self,
        task: T,
        priority: i32,
        dependencies: &[TaskId],
    ) -> TaskId {
        let id = self.next_task_id;
        self.next_task_id += 1;
        self.task_queue.lock().unwrap().push(Task {
            id,
            task: Box::new(task),
            priority,
        });
        if !dependencies.is_empty() {
            self.task_dependencies
                .lock()
                .unwrap()
                .insert(id, dependencies.to_vec());
        }
        id
    }

    pub fn cancel_task(&mut self, id: TaskId) -> bool {
        let mut queue = self.task_queue.lock().unwrap();
        if let Some(position) = queue.iter().position(|t| t.id == id) {
            queue.pop();
            true
        } else {
            false
        }
    }

    pub fn get_progress(&self) -> (usize, usize) {
        let completed = self.next_task_id - self.task_queue.lock().unwrap().len();
        (completed, self.next_task_id)
    }
}
