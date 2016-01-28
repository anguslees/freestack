@0xb47a25343d018380;

using import "/capnp/persistent.capnp".Persistent;
using Types = import "types.capnp";

using AuthUrl = Types.Url;

interface Auth {
  auth @0 (req :AuthRequest) -> (resp :AuthResponse);
  struct AuthRequest {
    method :union {
      token @0 :Data;

      password @1 :Password;
      # The password authentication method is used.
    }
    struct Password {
      user @0 :Text;
      # The user name.
      domain @1 :Text;
      # The name of the domain.
      password @2 :Text;
      # The password for the user.
    }
    scope :union {
      # (Optional) The authorization scope.
      #
      # (Since v3.4) Specify "unscoped" to make an explicit unscoped token request, which returns an unscoped response without any authorization. This request behaves the same as a token request with no scope where the user has no default project defined.
      #
      # If you do not make an explicit "unscoped" token request and your role has a default project, the response might return a project-scoped token. If a default project is not defined, a token is issued without an explicit scope of authorization, which is the same as asking for an explicit unscoped token.
      unspecified @2 :Void;
      unscoped @3 :Void;
      project @4 :Text;
      domain @5 :Text;
    }
  }
  struct AuthResponse {
    user @0 :User;
    roles @1 :List(Role);
    #catalog @2 :List(CatalogEntry);
    struct CatalogEntry {
      name @0 :Text;
      union {
        identity3 @1 :Identity;
        compute2 @2 :import "compute2.capnp".Compute;
      }
    }
  }
}

struct Role {
  name @0 :Text;
}

interface Domain { # extends(Persistent(Text), Types.Resource(Domain.Details))
  details @0 () -> (details :Details);
  struct Details {
    name @0 :Text;
    enabled @1 :Bool = true;
    description @2 :Text;
  }

  rootProject @1 () -> (project :Project);

  user @2 (name :Text) -> (user :User);
  users @3 (filter :UserFilter) -> (users :List(Text));  # FIXME: Should be List(User)
  struct UserFilter {
    name @0 :Text;
    enabled @1 :Bool = true;
  }

  group @4 (name :Text) -> (group :Group);
  groups @5 (filter :GroupFilter) -> (groups :List(Text));  # FIXME: Should be List(Group)
  struct GroupFilter {
    name @0 :Text;
  }
}

interface User {  # extends(Persistent(Text))
  # User entities represent individual API consumers and are owned by a specific domain.

  details @0 () -> (details :Details);
  struct Details {
    name @0 :Text;
    enabled @1 :Bool = true;
    email @2 :Text;
    password @3 :Text;
  }
}

interface Group {  # extends(Persistent(Text))
  # Group entities represent a collection of Users and are owned by a specific domain.

  details @0 () -> (details :Details);
  struct Details {
    name @0 :Text;
    description @1 :Text;
  }

  members @1 () -> (members :List(Text));  # FIXME: should be List(User)
}

interface Project { # extends(Persistent(Text))
  # Projects represent the base unit of “ownership” in OpenStack, in
  # that all resources in OpenStack should be owned by a specific
  # project (“projects” were also formerly known as “tenants”). A
  # project itself must be owned by a specific domain.

  details @0 () -> (details :Details);
  struct Details {
    name @0 :Text;
    # The project name.  The project can have the same name as its domain.

    description @1 :Text;
    # The project description.

    enabled @2 :Bool = true;
  }

  createSubProject @1 (details :Details) -> (project :Project);
  destroySubProject @2 (project :Project);
  updateSubProject @3 (project :Project, newDetails :Details);
}

interface Region {
  details @0 () -> (details :Details);
  struct Details {
    name @0 :Text;
    # The ID for the region.

    description @1 :Text;
    # The region description.
  }
}

interface Identity {

}
