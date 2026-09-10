"""Fail-closed additions to the frozen native adapters; compiler code is untouched."""


def replace_once(text, old, new):
    if text.count(old) != 1:
        raise ValueError("adapter patch anchor missing or ambiguous: " + repr(old[:100]))
    return text.replace(old, new)


def rust(source):
    source = replace_once(source, "    goroutines_ready: Option<usize>,\n", "    goroutines_ready: Option<usize>,\n    worker_timing: serde_json::Value,\n")
    source = replace_once(source, "    let ready = Barrier::new(workers + 1);", "    let timing_epoch = Instant::now();\n    let ready = Barrier::new(workers + 1);")
    source = replace_once(source, "        for _ in 0..workers {", "        for worker_index in 0..workers {")
    source = replace_once(source, "                let mut failure = None;\n                ready.wait();\n                while let Ok(index) = receive.recv() {", """                let mut failure = None;
                let mut received = Vec::with_capacity(file_count.div_ceil(workers));
                let mut work_ns = 0_u128;
                let mut receive_ns = 0_u128;
                let mut first_receive = None;
                ready.wait();
                let worker_start = timing_epoch.elapsed().as_nanos();
                loop {
                    let receive_started = Instant::now();
                    let receive_start_offset = receive_started.duration_since(timing_epoch).as_nanos();
                    let next = receive.recv();
                    let receive_ended = Instant::now();
                    receive_ns += receive_ended.duration_since(receive_started).as_nanos();
                    first_receive.get_or_insert((receive_start_offset, receive_ended.duration_since(timing_epoch).as_nanos()));
                    let Ok(index) = next else { break; };
                    received.push(index);""")
    source = replace_once(source, "                    let outcome = std::panic::catch_unwind", "                    let work_started = Instant::now();\n                    let outcome = std::panic::catch_unwind")
    source = replace_once(source, "                        Err(panic) => failure = Some(panic),\n                    }\n                }\n                finished.wait();", """                        Err(panic) => failure = Some(panic),
                    }
                    work_ns += work_started.elapsed().as_nanos();
                }
                let completed = timing_epoch.elapsed().as_nanos();
                finished.wait();""")
    source = replace_once(source, "                retained\n            })?);", "                (retained, (worker_index, received, work_ns, receive_ns, worker_start, completed, first_receive.expect(\"every worker attempts receive\")))\n            })?);")
    source = replace_once(source, "        let start = Instant::now();\n        for index in 0..file_count {\n            senders[index % workers].send(index)?;\n        }", """        let start = Instant::now();
        let pipeline_start = start.duration_since(timing_epoch).as_nanos();
        let mut sender_send_ns = 0_u128;
        for index in 0..file_count {
            let send_started = Instant::now();
            senders[index % workers].send(index)?;
            sender_send_ns += send_started.elapsed().as_nanos();
        }
        let sender_completed = start.elapsed().as_nanos();""")
    source = replace_once(source, "        let files: Vec<_> = handles", "        let (files, timing_rows): (Vec<_>, Vec<_>) = handles")
    source = replace_once(source, "            .collect();\n        let mut report = Report {", """            .unzip();
        // Assignment hashes and all JSON allocations happen after the retained endpoint.
        let worker_rows: Vec<_> = timing_rows.into_iter().map(|(worker, received, work_ns, receive_ns, worker_start, completed, (first_start, first_end))| {
            let pre_pipeline_receive = first_end.min(pipeline_start).saturating_sub(first_start);
            let mut digest = Sha256::new();
            digest.update(b"S07-worker-assignment-v1\\0");
            for &index in &received { digest.update((index as u64).to_be_bytes()); }
            serde_json::json!({
                "worker": worker, "files": received.len(),
                "loaded_bytes": received.iter().map(|&index| inputs[index].source.len()).sum::<usize>(),
                "assignment_sha256": format!("{:x}", digest.finalize()),
                "work_ns": work_ns, "receive_wait_ns": receive_ns - pre_pipeline_receive,
                "start_offset_ns": worker_start.saturating_sub(pipeline_start),
                "completion_offset_ns": completed - pipeline_start,
                "tail_ns": wall_time_ns - (completed - pipeline_start),
            })
        }).collect();
        let mut report = Report {""")
    source = replace_once(source, "            goroutines_ready: None,", """            goroutines_ready: None,
            worker_timing: serde_json::json!({
                "version": 1, "diagnostic_only": true, "runtime": "rust", "queue_capacity": workers,
                "assignment": "round_robin_index_mod_workers",
                "work_domain": "parse_bind_publication_validation_retention_elapsed",
                "wait_domain": "receive_call_elapsed_clipped_to_pipeline",
                "sender_domain": "send_call_elapsed_including_scheduling_and_queue_blocking",
                "sender_send_ns": sender_send_ns, "sender_completion_offset_ns": sender_completed,
                "rows": worker_rows,
            }),""")
    return source


def go(source):
    source = replace_once(source, 'type Report struct {', '''type WorkerTiming struct {
    Worker int `json:"worker"`
    Files int `json:"files"`
    LoadedBytes int `json:"loaded_bytes"`
    AssignmentSHA256 string `json:"assignment_sha256"`
    WorkNs int64 `json:"work_ns"`
    ReceiveWaitNs int64 `json:"receive_wait_ns"`
    StartOffsetNs int64 `json:"start_offset_ns"`
    CompletionOffsetNs int64 `json:"completion_offset_ns"`
    TailNs int64 `json:"tail_ns"`
}
type Report struct {
    WorkerTiming any `json:"worker_timing"`''')
    source = replace_once(source, '\tstart := make(chan struct{})', '''    start := make(chan struct{})
    timingRows := make([]WorkerTiming, workers)
    received := make([][]int, workers)
    var started time.Time''')
    source = replace_once(source, '\t\tgo func(worker int) {', '''        received[worker] = make([]int, 0, (len(inputs)+workers-1)/workers)
        go func(worker int) {''')
    source = replace_once(source, '\t\t\t<-start\n\t\t\tfor index := range channels[worker] {', '''            <-start
            row := &timingRows[worker]
            row.Worker = worker
            row.StartOffsetNs = time.Since(started).Nanoseconds()
            for {
                receiveStarted := time.Now()
                index, ok := <-channels[worker]
                row.ReceiveWaitNs += time.Since(receiveStarted).Nanoseconds()
                if !ok { break }
                received[worker] = append(received[worker], index)''')
    source = replace_once(source, '\t\t\t\tfunc() {', '                workStarted := time.Now()\n\t\t\t\tfunc() {')
    source = replace_once(source, '\t\t\t\t}()\n\t\t\t}\n\t\t\tdone.Done()', '''                }()
                row.WorkNs += time.Since(workStarted).Nanoseconds()
            }
            row.CompletionOffsetNs = time.Since(started).Nanoseconds()
            done.Done()''')
    source = replace_once(source, '\tstarted := time.Now()\n\tclose(start)\n\tfor index := range inputs {\n\t\tchannels[index%workers] <- index\n\t}', '''    started = time.Now()
    close(start)
    senderSendNs := int64(0)
    for index := range inputs {
        sendStarted := time.Now()
        channels[index%workers] <- index
        senderSendNs += time.Since(sendStarted).Nanoseconds()
    }
    senderCompleted := time.Since(started).Nanoseconds()''')
    source = replace_once(source, '\tfor worker, files := range results {', '''    // Hash and report allocation stay outside the retained measurement endpoint.
    for worker := range workers {
        row := &timingRows[worker]
        row.TailNs = elapsed.Nanoseconds() - row.CompletionOffsetNs
        row.Files = len(received[worker])
        digest := sha256.New()
        digest.Write([]byte("S07-worker-assignment-v1\\x00"))
        word := make([]byte, 8)
        for _, index := range received[worker] {
            row.LoadedBytes += len(inputs[index].text)
            binary.BigEndian.PutUint64(word, uint64(index))
            digest.Write(word)
        }
        row.AssignmentSHA256 = fmt.Sprintf("%x", digest.Sum(nil))
    }
    report.WorkerTiming = map[string]any{
        "version": 1, "diagnostic_only": true, "runtime": "go", "queue_capacity": workers,
        "assignment": "round_robin_index_mod_workers",
        "work_domain": "parse_bind_publication_validation_retention_elapsed",
        "wait_domain": "receive_call_elapsed_clipped_to_pipeline",
        "sender_domain": "send_call_elapsed_including_scheduling_and_queue_blocking",
        "sender_send_ns": senderSendNs, "sender_completion_offset_ns": senderCompleted,
        "rows": timingRows,
    }
    for worker, files := range results {''')
    return source
