# Jack Kingham


### Describe how your prefetcher works
---
I implemented a simple content directed prefetcher. This design was based off the one suggested in "A stateless, content-directed data prefetching mechanism." by Cooksey R. et al. This prefetcher looks through the memory that is being accessed to find addresses that are stored in that memory. Because the cachesim does not actually manage any memory, for each invocation of the prefetcher I randomly generate an array of line size bytes, and reinterpret each slice of 4 contiguous bytes as an address. The implementation proposed in the paper will determine if this address lies within one of the running process's memory segments, and prefetch the address if it does. Since the addresses used by the cachesim do not reflect real memory segments, I keep track of the minimum and maximum addresses seen by the prefetcher. If the address lies within this bound +- line size*N where N is a command line arg, then the address will be prefetched.


### Explain how you chose that prefetch strategy
---
For array accesses and such, it makes sense that the next memory access will be very spatially local to the previous. Different data structures like linked lists and trees often times do not exist in contiguous memory and therefore do not benefit from spatial locality. For such data structures, the only way to effectively prefetch the next memory access is to inspect the data using used to hopefully find the next pointer that will be dereferenced. I expected there to be a lot more publications investigating prefetching strategies that leverage pointer chains in memory, but this is surprisingly little I could find.

### Discuss the pros and cons of your prefetch strategy
---
**PROS**
This prefetching strategy is able to prefetch addresses that any other prefetcher would probably miss. Like I mentioned before, certain data structures like linked lists may have their nodes distributed throughout memory in random places. Some nodes may be next to eachother, while others may be in entirely different memory segments. The only way to successfully prefetch memory when traversing a linked list or other related datastructure is to find pointers in memory. This is not a super uncommon scenario, especially for kernel code.
**CONS**
This prefetching strategy is pretty computationally expensive. It spends a lot of time inspecting memory that doesn't contain pointers, all for the hope that it eventually does find a pointer. Additionally, there is a good chance that unrelated data, like a string" will resemble a pointer in memory, and therefore end up being prefetched despite being junk. In short, the strategy is expensive but not very intelligent, so it may waste time searching for and finding pointers that don't actually exist.

### Demonstrate that the prefetcher could be implemented in hardware
---
Like I mentioned before, a closely related idea has been proposed a long time ago: "A stateless, content-directed data prefetching mechanism." by Cooksey R. et al. Its not a very complicated prefetching strategy, theres no reason it couldn't be implemented in hardware.

### Cite any additional sources that you used to develop your prefetcher
I had the original memory inspection idea and used AI to hopefully find a research paper that proposes the same idea. "A stateless, content-directed data prefetching mechanism." by Cooksey R. et al.