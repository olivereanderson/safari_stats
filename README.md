**SAFARI_Stats**: A collection of CLI tools related to log file processing from an imagined popular safari app.

This is essentially my solution to a take home test I once had to do for a software engineering position which I received positive feedback on. I have done my utmost to rewrite the context and rename everything in order for the company who gave me the test to be reuse it. Apart from renaming variables and functions the code presented here is the same as my original submission. 

# Context:
Consider the following imaginary app: Users can participate in Safari *sessions*, which means finding 10 users in the vicinity that are split into two equally sized teams. Each session consists of several *trips* into a location nearby containing animals. The goal of each trip is to be the first team to take five pictures of animals (in total) before the other team (if both teams get their fifth picture simultaneously it is considered a draw). A trip can only last fifteen minutes and whichever team with the most pictures wins the trip. 

At the start of each trip every player has to rent a *camera* from a certified store. There are approximately a couple of hundred different cameras to choose from. 

At the end of each trip the participants upload the id of the camera they used together with how many pictures they took this trip. 

We assume that our server contains a folder that receives a text file names "safari-sessions-YYYYMMDD.log" at the end of each day. The rows of this file are of the following format 
```
user_id: The id of a user , session_id: The id of a session, camera_id: The id of a camera, nb_pics: The corresponding number of pictures taken in the given trip.
```
The former two fields both require 128 bits (they are UUID's) and the latter two can be represented by 8 bit integers. 

# Our task: 
1. At the end of each day: Produce the top one hundred average number of pictures per camera over the last seven day. 
The output must be a text file "camera_top100_YYYYMMDD.txt" where each row is of the form:
```
camera_id|session_id1:avg_pics1,session_id2:avg_pics2, ... , session_id100:avg_pics100
```
where avg_pics is the average number of pics by the given camera type in the corresponding session. 
2. Also at the end of each day: Provide an overview of the top ten number of pictures in sessions by users. 
The output must be a text file "user_top_10_YYYYMMDD.txt" where each row is of the form: 
```
user_id|session_id1:nb_pics1,session_id2:nb_pics2, ... , session_id10:nb_pics10
``` 
where nb_pics is the number of pictures taken by the corresponding user in the given session. 



# Included applications: 
There are three CLI tools contained in this collection. 

* camera-stats: Yields the top 100 average number of pictures by each camera over the last seven days. 
* user-stats: Yields the top 10 number of pictures in sessions over the last seven days fro each user. 
* Session-synthesiser: Generate session log files that can be used to test the two aforementioned programs. 

# Building 
First install Rust 1.53.0 or later ([installation instructions for Rust can be found here](https://www.rust-lang.org/learn/get-started)). Once Rust is installed run 
```bash 
$ cargo build --release
```
from the root of this directory. The executables can now be found in `./target/release`.

 # Usage 
 ## Generating synthetic session log files: 
 To generate seven days worth of session log files with say 250 000 sessions per day in the directory `./safari_synthetic_session_logs` simply run: 
 ```bash 
 $ cargo run --release --bin session-synthesiser -- ./safari_synthetic_session_logs --number-of-sessions 250000
 ```

## Compute the top one hundred average number of pictures by each camera:
To compute the top one hundred average number of pictures for each camera over the last seven days run the following command: 
```bash 
$ cargo run --release --bin camera-stats -- <directory containing safari session logs> <directory to store the resulting txt file>
```
Here is a concrete example: 
```bash
$  cargo run --release --bin camera-stats ./safari_synthetic_session_logs ./daily_camera_stats
```
This produces the file `./daily_camera_stats/camera_top100_YYYYMMDD.txt` where lines are of the form 
```
camera_id|session_id1:avg_pics1,session_id2:avg_pics2,..,session_id100:avg_pics100 
```

## Compute the top 10 sessions in terms of number of pictures by user
To compute the top 10 sessions in terms of number of pictures per user over the course of the last seven days run the following command: 
 
```bash
$ cargo run --release --bin user-stats -- <directory containing safari session logs> <directory to store the resulting txt file >
```
Here is a concrete example 
```bash
$ cargo run --release --bin user-stats ./safari_synthetic_session_logs ./daily_user_stats
```
which produces a file `./daily_user_stats/user_top_10_YYYYMMDD.txt` where lines are of the form 
```
user_id|session_id1:nb_pics1,session_id2:nb_pics2,...,session_id10:nb_pics10
```

## Overview of how the data processing programs (camera-stats and user-stats) work. 
Heuristically speaking camera-stats and user-stats are based on the same strategy, but their implementation details are rather different. 
The strategy goes as follows: 
1) parse and sort the records from a log file in batches (to keep memory consumption low). In the camera-stats case we sort lexicographically with respect to session id followed by camera id. In the user case we sort by user id followed by session id. 
2) compute the statistics we are interested in from the sorted records. 
3) Save the result of the previous computation for future reuse (note that we are doing this for one log file at a time, even though we are interested in the relevant statistics over the last seven days). 
4) repeat steps 1-3 until we have processed all log files in the last seven days (if some were already processed we can ignore them). 
5) Compute the statistics we are interested in over a seven day period from the stored files we produced in step 1-4 and write this to a text file that can be read by humans. 

The implementation for user-stats is different from camera-stats, because there are a lot more users than cameras and so it becomes trickier to study 
over a seven day period. In order to overcome this obstruction we use finite state transducers (see [this blog post for an explanation](https://blog.burntsushi.net/transducers/)) backed by memory maps. Note that reading from a memory mapped file is only safe if we can guarantee that nothing mutates the given file while we are reading from it. Another potential drawback is that memory mapped IO can be rather slow if our hard drive is not an SSD. 



## Memory usage 
Our programs do not require much RAM: camera-stats does definitely not use more than 1GB RAM (usually around 800-900 MB from what I have observed). With user-stats the story is a bit more complicated as we utilize memory maps in that case and thus it becomes harder to get an 
idea of the programs real memory consumption as the Maximum resident set size will share a lot of memory with the page cache. I did some experimenting and tried to run the program with a file that was 33 GB on disk. The Maximum resident set size (obtained from the `time -v` command) was at first rather big, but then I ran the program again while I was continously keeping the operating systems page cache busy by 
sending other large files to `/dev/null/: 
```
$ cat loads of large files > /dev/null
```
in this case the Maximum resident set size was around 1GB.  

## Possibilities for improvement 
If I had more time to work on this I would first of all have written a lot more tests. I would also have tried to improve the command line interfaces by adding progress bars (using the [indicatif crate](https://crates.io/crates/indicatif)). Furthermore I would have tried to make the user-stats package less imperative and tried to split the code up into more functions. Moreover I would have used [criterion](https://crates.io/crates/criterion) to add benchmarks so we could optimize our code to run faster in the future. Finally there are some (non performance sensitive places) where unnecessary copying has been done to quickly satisfy the borrow checker. Ideally this should also be refactored. 



