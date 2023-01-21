# Requirements

* Ability to Publish / Subscribe events
* Accept any arbitray type as a message
* Store messages that haven't been received by a subscriber (persistence to disk optional)
* Provide feedback to message sender about delivery status?

## Subscription Procedure

1. Spawn new spsc channel. Return rx side. Store tx side in `anymap<Vec<Sender<T>>>`

## Publish Procedure

1. Accept `Any` type as a message
2. See if a `Vec<T>` exists in the anymap for this type
3. If so, loop over the vec and send a clone of the message to each `Sender`
4. Report success or fail of message sent?
