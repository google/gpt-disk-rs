# Use cases

1. Create a whole disk from scratch. 
   * The disk will include both the partition table and the partition
     data.
   * The disk can be an in-memory slice or a file on disk.
   
2. Modify the partition layout of an existing disk.
   * The disk can be an in-memory slice or a file on disk.
   
3. Modify the partition data of an existing disk.
   * The disk can be an in-memory slice or a file on disk.

---

A hierarchical view:

1. First you establish the disk itself: block size, size in bytes, guid.
2. Then you establish the partition layout: number of partitions, LBA
   range for each partition.
3. Then you operate within those boundaries, allowing changes such as:
   * Read/write partition data.
   * Change disk GUID.
   * Change partition GUID/type GUID/attributes.

---

Idea: think of what interface we'd want for easymode when working from a
UEFI program. So, we have alloc and rng, but no std. What would the
interface look like then?
