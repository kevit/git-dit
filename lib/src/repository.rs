// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Repository related utilities
//!
//! This module provides the `RepositoryExt` extension trait which provides
//! issue handling utilities for repositories.
//!

use git2::{self, Commit, Oid, Tree};

use issue::Issue;
use error::*;
use error::ErrorKind as EK;
use iter;


/// Extension trait for Repositories
///
/// This trait is intended as an extension for repositories. It introduces
/// utility functions for dealing with issues, e.g. for retrieving references
/// for issues, creating messages and finding the initial message of an issue.
///
pub trait RepositoryExt {
    /// Retrieve an issue
    ///
    /// Returns the issue with a given id.
    ///
    fn find_issue(&self, id: Oid) -> Result<Issue>;

    /// Retrieve an issue by its head ref
    ///
    /// Returns the issue associated with a head reference.
    ///
    fn issue_by_head_ref(&self, head_ref: &git2::Reference) -> Result<Issue>;

    /// Find the issue with a given message in it
    ///
    /// Returns the issue containing the message provided
    ///
    fn issue_with_message<'a>(&'a self, message: &Commit<'a>) -> Result<Issue>;

    /// Get issue hashes for a prefix
    ///
    /// This function returns all known issues known to the DIT repo under the
    /// prefix provided (e.g. all issues for which refs exist under
    /// `<prefix>/dit/`). Provide "refs" as the prefix to get only local issues.
    ///
    fn issues_with_prefix(&self, prefix: &str) -> Result<iter::HeadRefsToIssuesIter>;

    /// Get all issue hashes
    ///
    /// This function returns all known issues known to the DIT repo.
    ///
    fn issues(&self) -> Result<iter::HeadRefsToIssuesIter>;

    /// Create a new issue with an initial message
    ///
    fn create_issue<'a, A, I, J>(&self,
             author: &git2::Signature,
             committer: &git2::Signature,
             message: A,
             tree: &Tree,
             parents: I
    ) -> Result<Issue>
        where A: AsRef<str>,
              I: IntoIterator<Item = &'a Commit<'a>, IntoIter = J>,
              J: Iterator<Item = &'a Commit<'a>>;

    /// Get an revwalk configured as a first parent iterator
    ///
    /// This is a convenience function. It returns an iterator over messages in
    /// reverse order, only following first parents.
    ///
    fn first_parent_revwalk(&self, id: Oid) -> Result<git2::Revwalk>;

    /// Get an IssueMessagesIter starting at a given commit
    ///
    /// The iterator returned will return messages in reverse order, following
    /// the first parent, starting with the commit supplied.
    ///
    fn issue_messages_iter<'a>(&'a self, commit: Commit<'a>) -> Result<iter::IssueMessagesIter<'a>>;

    /// Get an empty tree
    ///
    /// This function returns an empty tree.
    ///
    fn empty_tree(&self) -> Result<Tree>;
}

impl RepositoryExt for git2::Repository {
    fn find_issue(&self, id: Oid) -> Result<Issue> {
        let retval = Issue::new(self, id);

        // make sure the id refers to an issue by checking whether an associated
        // head reference exists
        if retval.heads()?.next().is_some() {
            Ok(retval)
        } else {
            Err(Error::from_kind(EK::CannotFindIssueHead(id)))
        }
    }

    fn issue_by_head_ref(&self, head_ref: &git2::Reference) -> Result<Issue> {
        let name = head_ref.name();
        name.and_then(|name| if name.ends_with("/head") {
                Some(name)
            } else {
                None
            })
            .and_then(|name| name.rsplitn(3, "/").nth(1))
            .ok_or_else(|| {
                let n = name.unwrap_or_default().to_owned();
                Error::from_kind(EK::MalFormedHeadReference(n))
            })
            .and_then(|hash| {
               Oid::from_str(hash)
                   .chain_err(|| EK::OidFormatError(hash.to_string()))
            })
            .map(|id| Issue::new(self, id))
    }

    fn issue_with_message<'a>(&'a self, message: &Commit<'a>) -> Result<Issue> {
        // follow the chain of first parents towards an initial message for
        // which a head exists
        for id in self.first_parent_revwalk(message.id())? {
            let issue = self.find_issue(id?);
            if issue.is_ok() {
                return issue
            }
        }

        Err(Error::from_kind(EK::NoTreeInitFound(message.id())))
    }

    fn issues_with_prefix(&self, prefix: &str) -> Result<iter::HeadRefsToIssuesIter> {
        let glob = format!("{}/dit/**/head", prefix);
        self.references_glob(&glob)
            .chain_err(|| EK::CannotGetReferences(glob))
            .map(|refs| iter::HeadRefsToIssuesIter::new(self, refs))
    }

    fn issues(&self) -> Result<iter::HeadRefsToIssuesIter> {
        let glob = "**/dit/**/head";
        self.references_glob(glob)
            .chain_err(|| EK::CannotGetReferences(glob.to_owned()))
            .map(|refs| iter::HeadRefsToIssuesIter::new(self, refs))
    }

    fn create_issue<'a, A, I, J>(&self,
             author: &git2::Signature,
             committer: &git2::Signature,
             message: A,
             tree: &Tree,
             parents: I
    ) -> Result<Issue>
        where A: AsRef<str>,
              I: IntoIterator<Item = &'a Commit<'a>, IntoIter = J>,
              J: Iterator<Item = &'a Commit<'a>>
    {
        let parent_vec : Vec<&Commit> = parents.into_iter().collect();

        self.commit(None, author, committer, message.as_ref(), tree, &parent_vec)
            .chain_err(|| EK::CannotCreateMessage)
            .map(|id| Issue::new(self, id))
            .and_then(|issue| {
                issue.update_head(issue.id())?;
                Ok(issue)
            })
    }

    fn first_parent_revwalk(&self, id: Oid) -> Result<git2::Revwalk> {
        self.revwalk()
            .and_then(|mut revwalk| {
                revwalk.push(id)?;
                revwalk.simplify_first_parent();
                revwalk.set_sorting(git2::SORT_TOPOLOGICAL);
                Ok(revwalk)
            })
            .chain_err(|| EK::CannotGetCommitForRev(id.to_string()))
    }

    fn issue_messages_iter<'a>(&'a self, commit: Commit<'a>) -> Result<iter::IssueMessagesIter<'a>> {
        iter::IssueMessagesIter::new(self, commit)
    }

    fn empty_tree(&self) -> Result<Tree> {
        self.treebuilder(None)
            .and_then(|treebuilder| treebuilder.write())
            .and_then(|oid| self.find_tree(oid))
            .chain_err(|| EK::CannotBuildTree)
    }
}

