extern crate uuid;
use std::sync::{Mutex, Arc};
use std::thread;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
struct Job {
    contractId: i32,
    id: uuid::Uuid,
    in_progress: bool
}

#[derive(Debug)]
struct Queue {
    job_list: Vec<Job>,
    workers: Vec<String>
}

impl Queue {
    fn process(&mut self) {
        // If the Queue is already in an active state, the jobs are already being handled, so
        // this instance of the call should do nothing. We only ignore this if we are being called
        // recursively from notify_next_in_line
        match self.job_list.last() {
            Some(job) => {
                if job.in_progress {
                    println!("We're already working on the most recent job, bailing out");
                    return;
                }
            },
            None => println!("Our queue is empty")
        }

        match self.job_list.len() > 0 {
            true => {
                self.notify_next_in_line();
            },
            false => {
                println!("Finished our queue, exiting");
            }
        }
    }

    fn notify_next_in_line (&mut self) {
        match self.job_list.last_mut() {
            Some(job) => {
                // We cant keep processing this queue, the last job is in progress
                if job.in_progress == true {
                    panic!("Oh no, this job shouldnt ever be hit a second time!!");
                }

                println!("Setting job to in progress {:?}", job);
                job.in_progress = true;
            },
            None => {
                panic!("Oh no, there should be work here!!");
            }
        }
    }

    fn add_to_queue (&mut self, job: Job) {
        self.job_list.insert(0, job);
        println!("{:?}", self.job_list);
        self.process();
    }

    fn job_complete(&mut self, job_id: uuid::Uuid) {
        println!("{:?}", self.job_list);
        match self.job_list.pop() {
            Some(job) => {
                if job.id != job_id {
                    println!("{:?}", job);
                    println!("{:?}", job_id);
                    panic!("Oh no, the queue is broken, something is out of order!!");
                }

                self.process();
                println!("finished job {:?}", job_id);
            },
            None => panic!("Job complete, yet there are no jobs in the list")
        }
    }
}

fn main() {
    let queue = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];

    // Ids for testing
    let ids: Vec<uuid::Uuid> = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    // Mock HTTP additions to queue
    for x in 0..3 {
        let q = queue.clone();
        let i = ids.clone();

        let handle = thread::spawn(move || {
            let mut q_lock = q.lock().unwrap();
            let stat = q_lock.entry("job_one").or_insert(Queue {
                job_list: vec![],
                workers: vec![],
            });

            println!("{:?}", x);
            stat.add_to_queue(Job {contractId: x, id: i[x as usize], in_progress: false});
        });

        handles.push(handle);
    }

    // Mock HTTP completions of queue, the problem here is we dont know the order things were pushed
    for x in 0..3 {
        let q = queue.clone();
        let i = ids.clone();
        thread::sleep_ms(10);

        let handle = thread::spawn(move || {
            let mut q_lock = q.lock().unwrap();

            match q_lock.get_mut("job_one") {
                Some(stat) => {
                    stat.job_complete(i[x as usize]);
                },
                None => panic!("Oh no, we should have a queue here")
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {:?}", *queue.lock().unwrap());
}
