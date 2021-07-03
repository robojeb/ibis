# Ibis: Turning Userland into "Me"-serland

Ibis is a project to learn about the Linux userland and all of 
its various components. I intend to document my learning (including mistakes) 
so other curious people can also learn about the Linux userland. 
For each step I take in understanding I will try to write a blog-post/tutorial
explaining what I learned and how someone else can recreate it. 




# Tutorials
## A Barebones System
1. [Getting set-up and the world's worst init](tutorials/01_setup_and_bad_init.html)
1. [Going Interactive](tutorials/02_going_interactive.html)

# FAQ

## Why call it "Ibis"

I had three criteria:

1. An animal (because a Gnu is an animal)
1. A google search doesn't immediately turn up another project
1. Bacronym potential

The Ibis is a pretty cool bird which covers these three criteria. 


## Why Rust

Because I like it. I enjoy programming in Rust. 
I didn't really consider any of the safety aspects when making this decision. 
The use of `unsafe` is also nice from a teaching perspective because it helps
more easily show where things could go horribly wrong. 
