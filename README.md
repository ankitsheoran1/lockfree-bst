# lockfree-bst
Time Complexity of each Operation is O(H(n) + c)
H(n) = height of BST 
c is contentions 

##
Modify Operation after helping a concurrent modify operation restart not from root rather  from a level of vicinity of failure. 


# Strive to exploit maximum possible Disjoint access Parllelism. 
(Disjoint Access Parallelism (DAP) is a concept in computer architecture and parallel computing that refers to a scenario where multiple processors or threads access different sets of memory locations simultaneously, without any overlap or contention for the same memory locations. In other words, DAP occurs when different parts of a program or computation can be executed in parallel without requiring synchronization or coordination between the processors or threads.)

Helping mechanism which ensures non-blocking progress may prove counterproductive to the performance if not used judiciously. However, in some situations the proportion of Remove operations may increase and they may need help to finish their pending steps. Then it is better to help them so that the traversal path does not contain large number of “under removal” nodes. Keeping that in view, we take helping to a level of adaptivity to the read-write load. In our algorithm, one may choose whether an operation helps a concurrent Remove operation during its traversal. We believe that this adaptive conservative helping in internal BSTs may be very useful in some situations. This is a useful contribution of this work.

To implement a lock-free BST, we represent it in a threaded format, right-child pointer, if null, is threaded and is used to point to the successor node, whereas a similar left-child pointer is threaded and pointed to the node itself. In this representation a binary tree can be viewed as an ordered list with exactly two outgoing and two incom- ing pointers per node.

A single pointer needs to be mod- ified to Add a node in the list.To Remove a node we may have to modify up to four pointers.In our design an operation restarts from a node at the vicinity of the link where it fails, after the required helping. To achieve that, we need to get hold of the appropriate node(s) to restart at. For that, we use a backlink per node and ensure that it points to a node present in the tree from where the failure spot is a single link away. It should be noted that a backlink is not used for the tree traversal.
To remove a binary node we replace it with its predecessor, and hence, the incoming and outgoing links of the predecessor also need to be modified in addition to the incoming links of the node itself. In order to remove a node, unlike traditional categorization of nodes of a BST into leaf, unary and binary, we categorize them into three categorie
Nodes belonging to category 1 are those whose order-link emanates from themselves; for a category 2 node, it emanates from its left-child; and for a category 3 node the incoming order-link emanates from the rightmost node in its left-subtree
Note that, the order-node of a category 1 node is the node itself, whereas for category 2 and category 3 nodes it is its predecessor.

To remove a node of category 1, only the incoming parent- link needs to be modified to connect to the node pointed by the right-link. 
For a category 2 node, the parent-link is up- dated to connect to the node pointed by the left-link and the order-link is modified to point to the node which the right-link was pointing to.
In order to remove a category 3 node, its predecessor replaces it and the incoming and out- going links of the predecessor are updated to take the values of that of the removed node. Parent-link of the predeces- sor is connected to the node which its left-link was pointing to before it got shifted.

The flag-mark order of links for a category 3 node is as following. 
1. flag the incoming order-link 
2. set the prelink
3. mark the outgoing right-link 
4. flag the parent-link of the predecessor incoming to that
5. flag the incoming parent- link
6. mark the outgoing left-link and finally
7. mark the outgoing left-link of the predecessor

For a node belong- ing to category 1 or 2, because there is no node between its order-node and itself, steps (IV), (VI) and (VII) do not hap- pen.

Because we follow orderly modifications of the links, it never allows a node to be missed by a traversal in the BST unless both its incoming links are pointed away.
However, because a node may shift “upward” to replace its successor, the interval associated with the order-link of its successor may shift “rightward” i.e the right subtree of the node after the successor is removed. Therefore, in order to termi- nate a traversal correctly, we use the stopping criterion given in Condition 1




We use two dummy nodes as global variables represented by a two member array of node- pointers called root. The keys −∞ and ∞ are stored in the two members of root and they can never be deleted. Node root[0] is left-child and predecessor of the node root[1]

# Locate Node 
The return value of Locate can be 0, 1 or 2 depending on whether the key k is less than, greater than or equal to the key kcurr at the termi- nation point, line 10. If kcurr̸=k then the desired interval is associated with the threaded outgoing link from x(kcurr) in the direction indicated by return value of Locate - 0 de- notes left and 1 denotes right. The termination criterion for Locate implements Condition 1.
we can make a traversal eagerly help pending Remove operations, even though it is not ob- structed by them, in the situations in which proportion of Remove increases. If that is done then a traversal cleans a node whose marked right-link it encounters during execu- tion by calling the function CleanMark.This functionality can be enabled using a boolean variable as an input argu- ment to every Set operation which is further passed to the Locate that it performs.

To Remove a node x(k), starting from {x(∞), x(−∞)} we locate the link that the interval containing the key (k − ε) associates with, see line 33. If the Locate terminates at {x(kprev ), x(kcurr )} then suc(x(kcurr )) is the desired node to remove if k matches with its key. suc(x(kcurr)) is the node pointed by the threaded right-link of x(kcurr) and this link is indeed the order-link of x(k). If x(k) is located then we try to flag its order-link using TryFlag,  in order to perform the step (I) of Remove.