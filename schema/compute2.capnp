@0xeda757bd6f27ff48;

using import "/capnp/persistent.capnp".Persistent;
using Types = import "types.capnp";
using Identity = import "identity3.capnp";

using Tenant = Identity.Domain;
using Iso8601Date = Types.Iso8601Date;
using AuthUrl = Types.Url;
using Metadata = Types.Metadata;

interface Compute {
  details @0 () -> (details :Details);
  struct Details {
    name @0 :Text;
  }

  limits @1 () -> (limits: List(Limit));
  struct Limit {
    absolute :group {
      # An absolute limit value of -1 indicates that the absolute limit for the item is infinite.

      maxImageMeta @0 :Int16 = -1;
      # The maximum number of key-value pairs per image for the project.

      maxPersonality @1 :Int16 = -1;
      # The maximum number of file path and content pairs that can be
      # supplied on server build and rebuild for the project.

      maxPersonalitySize @2 :Int64 = -1;
      # The maximum size, in bytes, for each personality file that is
      # guaranteed to apply to all images in the project. Providers
      # can set additional per-image personality limits.

      maxSecurityGroupRules @3 :Int16 = -1;
      # The maximum number of security group rules per security group for the project.

      maxSecurityGroups @4 :Int16 = -1;
      # The maximum number of security groups per server for the project.

      maxServerMeta @5 :Int16 = -1;
      # The maximum number of metadata key-value pairs that can be supplied per server for the project.

      maxTotalCores @6 :Int64 = -1;
      # The maximum number of cores for the project.

      maxTotalFloatingIps @7 :Int64 = -1;
      # The maximum number of floating IP addresses for the project.
      #
      # These IPs must be allocated from the central pool before you
      # can use them. To allocate a floating IP to a project, see
      # Associate floating IP addresses.

      maxTotalInstances @8 :Int64 = -1;
      # The maximum number of servers at any one time for the project.

      maxTotalKeypairs @9 :Int16 = -1;
      # The maximum number of key pairs per server for the project.

      maxTotalRAMSize @10 :Int64 = -1;
      # The maximum total amount of RAM, in MB, for all servers at any one time for the project.

      maxServerGroups @11 :Int16 = -1;
      # The maximum number of server groups per server for the project.

      maxServerGroupMembers @12 :Int32 = -1;
      # The maximum number of server group members per server group for the project.
    }

    rate @13 :List(RateLimit);
    # Current rate limits.
    struct RateLimit {
      limit @0 :List(ApiLimit);
      struct ApiLimit {
	nextAvailable @0 :Text;
	remaining @1 :UInt32;
	unit @2 :Text;
	value @3 :UInt32;
	verb @4 :Text;
      }

      regex @1 :Text;
      uri @2 :Text;
    }
  }

  servers @2 (filter: ServersFilter) -> (servers: List(Server.Details));  # FIXME
  struct ServersFilter {
    id @0 :Text;

    name @1 :Text;
    # The name of the server as a string. Can be queried with regular
    # expressions. The regular expression ?name=bob returns both bob
    # and bobb. If you must match on only bob, you can use a regular
    # expression that matches the syntax of the underlying database
    # server that is implemented for Compute, such as MySQL or
    # PostgreSQL.

    tenant @2 :Tenant;
    # The tenant ID in a multi-tenancy cloud.

    changesSince @3 :Iso8601Date;
    # The date and time when the image or server last changed status.
    #
    # Use this query parameter to check for changes since a previous
    # request rather than re-downloading and re-parsing the full
    # status at each polling interval. If data has changed, only the
    # items changed since the specified time are returned in the
    # response. If data has not changed since the changes-since time,
    # an empty list is returned.
    #
    # For example, issue a GET request against the following endpoint
    # to list all servers that have changed since Mon, 24 Jan 2015
    # 17:08:00 UTC:
    #
    # GET /v2/010101/servers?changes-since=2015-01-24T17:08:00Z
    #
    # To enable you to keep track of changes, this filter also
    # displays images and servers that were deleted if the
    # changes-since value specifies a date in the last 30 days. Items
    # deleted more than 30 days ago might be returned, but it is not
    # guaranteed.

    image @4 :Image;
    # The UUID for the image.

    flavor @5 :Flavor;
    # The UUID for the specific flavor, which is a combination of memory, disk size and CPUs.

    status @6 :Server.Status;

    host @7 :Text;
    # Name of the host as a string.

    limit @8 :UInt32 = 0;
    marker @9 :Text;
  }

  flavors @3 (filter: FlavorsFilter) -> (flavors: List(Flavor.Details));  # FIXME
  # List available flavors.
  struct FlavorsFilter {
    minDisk @0 :UInt64;
    minRam @1 :UInt64;
  }
}

interface Server {  # extends(Persistent(Text))
  enum Status {
    # Servers contain a status attribute that indicates the current server state. You can filter on the server status when you complete a list servers request. The server status is returned in the response body. The possible server status values are:

    active @0;
    # The server is active.

    building @1;
    # The server has not finished the original build process.

    deleted @2;
    # The server is permanently deleted.

    error @3;
    # The server is in error.

    hardReboot @4;
    # The server is hard rebooting. This is equivalent to pulling the power plug on a physical server, plugging it back in, and rebooting it.

    password @5;
    # The password is being reset on the server.

    paused @6;
    # In a paused state, the state of the server is stored in RAM. A paused server continues to run in frozen state.

    reboot @7;
    # The server is in a soft reboot state. A reboot command was passed to the operating system.

    rebuild @8;
    # The server is currently being rebuilt from an image.

    rescued @9;
    # The server is in rescue mode. A rescue image is running with the original server image attached.

    resized @10;
    # Server is performing the differential copy of data that changed during its initial copy. Server is down for this stage.

    revertResize @11;
    # The resize or migration of a server failed for some reason. The destination server is being cleaned up and the original source server is restarting.

    softDeleted @12;
    # The server is marked as deleted but the disk images are still available to restore.

    stopped @13;
    # The server is powered off and the disk image still persists.

    suspended @14;
    # The server is suspended, either by request or necessity. This status appears for only the following hypervisors: XenServer/XCP, KVM, and ESXi. Administrative users may suspend an instance if it is infrequently used or to perform system maintenance. When you suspend an instance, its VM state is stored on disk, all memory is written to disk, and the virtual machine is stopped. Suspending an instance is similar to placing a device in hibernation; memory and vCPUs become available to create other instances.

    unknown @15;
    # The state of the server is unknown. Contact your cloud provider.

    verifyResize @16;
    # System is awaiting confirmation that the server is operational after a move or resize.
  }

  details @0 () -> (details :Details);
  struct Details {
    id @0 :Text;
    name @1 :Text;
    tenant @2 :Tenant;

    status @3 :Status;

    flavor @4 :Flavor;
    image @5 :Image;

    progress @6 :Float32;

    user @7 :Identity.User;

    created @8 :Iso8601Date;
    updated @9 :Iso8601Date;

    metadata @10 :Metadata;

    hostId @11 :Text;

    accessIPV4 @12 :Text;
    accessIPV6 @13 :Text;

    addresses @14 :List(Network);
    struct Network {
       name @0 :Text;

       addresses @1 :List(Address);
       struct Address {
         union {
	   addr4 @0 :Text;
	   addr6 @1 :Text;
	 }
       }
    }
  }

  update @1 (updated :UpdateDetails) -> (details :Details);
  # Updates the editable attributes of the specified server.
  struct UpdateDetails {
    name @0 :Text;
    # The name of the server. If you edit the server name, the server host name does not change. Also, server names are not guaranteed to be unique.

    accessIPv4 @1 :Text;
    # The IP version 4 address.
    accessIPv6 @2 :Text;
    # The IP version 6 address.
    autoDiskConfig @3 :Bool;
    # Defines how the server's root disk partition is handled when the server is started or resized. Valid values are:
    #
    # True. The root partition expands to encompass all available virtual disks.
    #
    # False. The root partition remains the same size as the original image. The operating system sees extra virtual disk space as unformatted free space.
  }

  delete @2 () -> (result :DeleteResult);
  # Deletes a specified server.
  struct DeleteResult {
    union {
      ok @0 :Void;
      err @1 :Text;
    }
  }

  changePassword @3 (adminPass :Text) -> (result :ChangePasswordResult);
  # Changes the password for a server.
  struct ChangePasswordResult {
    union {
      ok @0 :Void;
      err @1 :Text;
    }
  }

  reboot @4 (type: RebootType) -> (result :RebootResult);
  # Reboots the specified server`
  enum RebootType {
    # The type of reboot.
    soft @0;
    # Signal the operating system to restart.
    hard @1;
    # Restart the server. Equivalent to power cycling the server.
  }
  struct RebootResult {
    union {
      ok @0 :Void;
      err @1 :Text;
    }
  }

  rebuild @5 (opts :RebuildOpts) -> (details :Details);
  # Rebuilds the specified server.
  struct RebuildOpts {
    image @0 :Image;
    adminPass @1 :Text;
  }

  resize @6 (flavor :Flavor) -> (result :ResizeResult);
  # Resizes the specified server.
  struct ResizeResult {
    union {
      ok @0 :Void;
      err @1 :Text;
    }
  }

  createImage @7 (opts :CreateImageOpts) -> (result :Types.Result(Image, Text));
  # Create an image.
  struct CreateImageOpts {
    name @0 :Text;
    # Name of the snapshot.
    metadata @1 :Metadata;
    # Metadata key and value pairs.
  }
}

interface Flavor { # extends(Persistent(Text))
  # A flavor is a hardware configuration for a server. Each flavor is
  # a unique combination of disk space and memory capacity.

  details @0 () -> (details :Details);
  struct Details {
    name @0 :Text;
    disk @1 :UInt64;
    # Disk space in GB.
    ram @2 :UInt64;
    # RAM in MB.
    vcpus @3 :UInt16;
  }
}

interface Image { #  extends(Persistent(Text))
  # An image is a collection of files that you use to create and
  # rebuild a server.  By default, operators provide pre-built
  # operating system images.  You can also create custom images: See
  # Server actions.

  enum Status {
    queued @0;
    # The Image service reserved an image ID for the image in the
    # registry but did not yet upload any image data.

    saving @1;
    # The Image service is currently uploading the raw data for the image.

    active @2;
    # The image is active and fully available in the Image service.

    killed @3;
    # An image data upload error occurred.

    deleted @4;
    # The Image service retains information about the image but the
    # image is no longer available for use.

    pendingDelete @5;
    # Similar to the deleted status. An image in this state is not recoverable.
  }

  details @0 () -> Details;
  struct Details {
    name @0 :Text;
    status @1 :Status;
    type @2 :Text;
    server @3 :Server;
    created @4 :Iso8601Date;
    updated @5 :Iso8601Date;
    progress @6 :Float32;
    minDisk @7 :UInt64;
    minRam @8 :UInt64;
    metadata @9 :Metadata;
  }

  delete @1 ();
}
