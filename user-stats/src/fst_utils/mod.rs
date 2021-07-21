// This module contains all the functionality concerning the use of finite state transducers (fst)s in this crate.
// In particular it enabels us to : 1) create an fst Set corresponding to the top 10 pics in sessions by user
// from a log file on each day.
// And 2) take the union of the stored fst Sets, thus enabling us to write a text file containing the top 10 number of pics in sessions by user
// over several days.
//
// IMPORTANT REMARK: The submodules of this module are coupled as follows:
// batching::from_log_file_to_batched_fst_maps stores fst maps in a specified folder for temporary fst maps.
// storing::from_batched_fst_maps_to_fst_set loads the aforementioned fst maps and takes their union. From this union a set of
// the top 10 number of pics in sessions by user is stored as an fst set. Where the keys have a very particular encoding that is crucial to
// finalizing::from_fst_sets_to_stats_file.
pub(crate) mod batching;
pub(crate) mod finalizing;
pub(crate) mod storing;
