# 功能实现

linkat的实现思路是在`ROOT_INODE`添加一个新的`DirEntry`，`name`为`newpath`,`inode_id`为`oldpath`的`inode_id`。

unlinkat的实现思路是删除`path`对应的`DirEntry`。如果没有其他的`DirEntry`的`inode_id`等于`path`的`inode_id`，则回收该`inode_id`的对应的数据块。

fastat的实现思路是查找`fd`对应的`inode_id`，`nlink`就是共享该`inode_id`的`DirEntry`数量，`mode`也根据`inode`判断。

# 问答作业

`root inode`相当于整个文件系统的根`/`。如果它内容损毁，系统将无法`ls`根目录下的文件。