#!/bin/bash
~/repos/maelstrom/maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10
~/repos/maelstrom/maelstrom test -w unique-ids --bin target/debug/unique_ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
~/repos/maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition
~/repos/maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 25 --time-limit 20 --rate 100
~/repos/maelstrom/maelstrom test -w g-counter --bin target/debug/grow_only_counter --node-count 3 --rate 100 --time-limit 20 --nemesis partition

