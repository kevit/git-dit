% GIT-DIT(1) User Manuals
% Matthias Beyer, Julian Ganz
% June 22, 2017

# NAME

git-dit - the distributed issue tracker for git


# SYNOPSIS

git [git options] dit \<command\> [\<args\>]


# DESCRIPTION

Git-dit is a distributed issue tracking system based on the git revision
control system.
Following some of the design principles of git, it offers both high level
commands for end users as well as a low level interface to internals for use in
scripts or third-party extensions.

For more information, refer to the DISCUSSION section of this manual.


# OPTIONS

There are no git-dit specific global options, contrary to sub-commands of
git-dit.
The `-h` short option usually prints a short help message listing all options
and command line arguments accepted by a specific sub-command.


# GIT-DIT COMMANDS

Like git, the git-dit suite is divided into high level ("porcelain") commands
and low level ("plumbing") commands.


# HIGH LEVEL COMMANDS (PORCELAIN)

## git-dit-new
    Add a new issue.

## git-dit-reply
    Reply to an existing issue with a new message.

## git-dit-show
    Show the contents (messages) of an issue.

## git-dit-list
    List all issues known to git-dit in the current directory.

## git-dit-tag
    Show or modify meta-data of issues.

## git-dit-fetch
    Fetch issues from a remote repository.

## git-dit-push
    Push issues to a remote repository.


# LOW LEVEL COMMANDS (PLUMBING)

## git-dit-check-message
    Check whether the format of an issue message is valid.

## git-dit-create-message
    Create a bare message.

## git-dit-find-tree-init-hash
    Find the issue hash for a message's hash.

## git-dit-get-issue-tree-init-hashes
    List all known issue hashes.

## git-dit-get-issue-metadata
    Extract meta-data from a thread in an issue.


# DISCUSSION

Git-dit is modeled after classical mailing lists.
Issue messages, which are stored as commits in the repository, correspond to
Emails.
Each issue can be thought of as a discussion thread on a mailing list, whether
it be a bug-report, a feature request or a patch-set to be merged into some
branch.

The command `git-dit-new` creates a new issue, or discussion thread.
It can be either based on an existing commit, or as a new initial commit.
The latter one is most appropriate for feature requests and bug-reports, while
the former one is especially useful for patch-sets.
Discussion messages will be created using `git-dit-reply` as empty commits,
preserving the parent commit's tree.


# SEE ALSO

