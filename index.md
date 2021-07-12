# Ibis: Turning Userland into "Me"-serland

Ibis is a project to learn about the Linux userland and all of 
its various components. I intend to document my learning (including mistakes) 
so other curious people can also learn about the Linux userland. 
For each step I take in understanding I will try to write a blog-post/tutorial
explaining what I learned and how someone else can recreate it. 




# Posts
## A Barebones System
1. [Getting set-up and the world's worst init](posts/01_setup_and_bad_init.html)
1. [Going Interactive](posts/02_going_interactive.html)
1. [Cleanup on PID 1](posts/03_cleanup_on_pid_one.html)

# FAQ

## Why call it "Ibis"

I had three criteria:

1. An animal (because a Gnu is an animal)
1. A google search doesn't immediately turn up another project
1. Bacronym potential

The Ibis is a pretty cool bird which covers these three criteria. 
Regarding the backronym I have been saying it stands for 
"I built it m'Self". 

## Why Rust

Because I like it. I enjoy programming in Rust. 
I didn't really consider any of the safety aspects when making this decision. 
The use of `unsafe` is also nice from a teaching perspective because it helps
more easily show where things could go horribly wrong. 

## What level of programming do I need to follow these posts

From time to time in the posts I will dig into Rust concepts, but I expect
most people to be familiar with: 

* Basic Rust project structure
* Including and using libraries
* Basic Error handling with Result
* Basic generics

