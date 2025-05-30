/* eslint-disable */
import { MerklePrefix } from "../../commitment/v1/commitment";
import { BinaryReader, BinaryWriter } from "../../../../binary";
import { isSet, DeepPartial, Exact } from "../../../../helpers";
export const protobufPackage = "ibc.core.connection.v1";
/**
 * State defines if a connection is in one of the following states:
 * INIT, TRYOPEN, OPEN or UNINITIALIZED.
 */
export enum State {
  /** STATE_UNINITIALIZED_UNSPECIFIED - Default State */
  STATE_UNINITIALIZED_UNSPECIFIED = 0,
  /** STATE_INIT - A connection end has just started the opening handshake. */
  STATE_INIT = 1,
  /**
   * STATE_TRYOPEN - A connection end has acknowledged the handshake step on the counterparty
   * chain.
   */
  STATE_TRYOPEN = 2,
  /** STATE_OPEN - A connection end has completed the handshake. */
  STATE_OPEN = 3,
  UNRECOGNIZED = -1,
}
export function stateFromJSON(object: any): State {
  switch (object) {
    case 0:
    case "STATE_UNINITIALIZED_UNSPECIFIED":
      return State.STATE_UNINITIALIZED_UNSPECIFIED;
    case 1:
    case "STATE_INIT":
      return State.STATE_INIT;
    case 2:
    case "STATE_TRYOPEN":
      return State.STATE_TRYOPEN;
    case 3:
    case "STATE_OPEN":
      return State.STATE_OPEN;
    case -1:
    case "UNRECOGNIZED":
    default:
      return State.UNRECOGNIZED;
  }
}
export function stateToJSON(object: State): string {
  switch (object) {
    case State.STATE_UNINITIALIZED_UNSPECIFIED:
      return "STATE_UNINITIALIZED_UNSPECIFIED";
    case State.STATE_INIT:
      return "STATE_INIT";
    case State.STATE_TRYOPEN:
      return "STATE_TRYOPEN";
    case State.STATE_OPEN:
      return "STATE_OPEN";
    case State.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}
/**
 * ConnectionEnd defines a stateful object on a chain connected to another
 * separate one.
 * NOTE: there must only be 2 defined ConnectionEnds to establish
 * a connection between two chains.
 */
export interface ConnectionEnd {
  /** client associated with this connection. */
  client_id: string;
  /**
   * IBC version which can be utilised to determine encodings or protocols for
   * channels or packets utilising this connection.
   */
  versions: Version[];
  /** current state of the connection end. */
  state: State;
  /** counterparty chain associated with this connection. */
  counterparty: Counterparty;
  /**
   * delay period that must pass before a consensus state can be used for
   * packet-verification NOTE: delay period logic is only implemented by some
   * clients.
   */
  delay_period: bigint;
}
/**
 * IdentifiedConnection defines a connection with additional connection
 * identifier field.
 */
export interface IdentifiedConnection {
  /** connection identifier. */
  id: string;
  /** client associated with this connection. */
  client_id: string;
  /**
   * IBC version which can be utilised to determine encodings or protocols for
   * channels or packets utilising this connection
   */
  versions: Version[];
  /** current state of the connection end. */
  state: State;
  /** counterparty chain associated with this connection. */
  counterparty: Counterparty;
  /** delay period associated with this connection. */
  delay_period: bigint;
}
/** Counterparty defines the counterparty chain associated with a connection end. */
export interface Counterparty {
  /**
   * identifies the client on the counterparty chain associated with a given
   * connection.
   */
  client_id: string;
  /**
   * identifies the connection end on the counterparty chain associated with a
   * given connection.
   */
  connection_id: string;
  /** commitment merkle prefix of the counterparty chain. */
  prefix: MerklePrefix;
}
/** ClientPaths define all the connection paths for a client state. */
export interface ClientPaths {
  /** list of connection paths */
  paths: string[];
}
/** ConnectionPaths define all the connection paths for a given client state. */
export interface ConnectionPaths {
  /** client state unique identifier */
  client_id: string;
  /** list of connection paths */
  paths: string[];
}
/**
 * Version defines the versioning scheme used to negotiate the IBC verison in
 * the connection handshake.
 */
export interface Version {
  /** unique version identifier */
  identifier: string;
  /** list of features compatible with the specified identifier */
  features: string[];
}
/** Params defines the set of Connection parameters. */
export interface Params {
  /**
   * maximum expected time per block (in nanoseconds), used to enforce block delay. This parameter should reflect the
   * largest amount of time that the chain might reasonably take to produce the next block under normal operating
   * conditions. A safe choice is 3-5x the expected time per block.
   */
  max_expected_time_per_block: bigint;
}
function createBaseConnectionEnd(): ConnectionEnd {
  return {
    client_id: "",
    versions: [],
    state: 0,
    counterparty: Counterparty.fromPartial({}),
    delay_period: BigInt(0)
  };
}
export const ConnectionEnd = {
  typeUrl: "/ibc.core.connection.v1.ConnectionEnd",
  encode(message: ConnectionEnd, writer: BinaryWriter = BinaryWriter.create()): BinaryWriter {
    if (message.client_id !== "") {
      writer.uint32(10).string(message.client_id);
    }
    for (const v of message.versions) {
      Version.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    if (message.state !== 0) {
      writer.uint32(24).int32(message.state);
    }
    if (message.counterparty !== undefined) {
      Counterparty.encode(message.counterparty, writer.uint32(34).fork()).ldelim();
    }
    if (message.delay_period !== BigInt(0)) {
      writer.uint32(40).uint64(message.delay_period);
    }
    return writer;
  },
  decode(input: BinaryReader | Uint8Array, length?: number): ConnectionEnd {
    const reader = input instanceof BinaryReader ? input : new BinaryReader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseConnectionEnd();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.client_id = reader.string();
          break;
        case 2:
          message.versions.push(Version.decode(reader, reader.uint32()));
          break;
        case 3:
          message.state = (reader.int32() as any);
          break;
        case 4:
          message.counterparty = Counterparty.decode(reader, reader.uint32());
          break;
        case 5:
          message.delay_period = reader.uint64();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },
  fromJSON(object: any): ConnectionEnd {
    const obj = createBaseConnectionEnd();
    if (isSet(object.client_id)) obj.client_id = String(object.client_id);
    if (Array.isArray(object?.versions)) obj.versions = object.versions.map((e: any) => Version.fromJSON(e));
    if (isSet(object.state)) obj.state = stateFromJSON(object.state);
    if (isSet(object.counterparty)) obj.counterparty = Counterparty.fromJSON(object.counterparty);
    if (isSet(object.delay_period)) obj.delay_period = BigInt(object.delay_period.toString());
    return obj;
  },
  toJSON(message: ConnectionEnd): unknown {
    const obj: any = {};
    message.client_id !== undefined && (obj.client_id = message.client_id);
    if (message.versions) {
      obj.versions = message.versions.map(e => e ? Version.toJSON(e) : undefined);
    } else {
      obj.versions = [];
    }
    message.state !== undefined && (obj.state = stateToJSON(message.state));
    message.counterparty !== undefined && (obj.counterparty = message.counterparty ? Counterparty.toJSON(message.counterparty) : undefined);
    message.delay_period !== undefined && (obj.delay_period = (message.delay_period || BigInt(0)).toString());
    return obj;
  },
  fromPartial<I extends Exact<DeepPartial<ConnectionEnd>, I>>(object: I): ConnectionEnd {
    const message = createBaseConnectionEnd();
    message.client_id = object.client_id ?? "";
    message.versions = object.versions?.map(e => Version.fromPartial(e)) || [];
    message.state = object.state ?? 0;
    if (object.counterparty !== undefined && object.counterparty !== null) {
      message.counterparty = Counterparty.fromPartial(object.counterparty);
    }
    if (object.delay_period !== undefined && object.delay_period !== null) {
      message.delay_period = BigInt(object.delay_period.toString());
    }
    return message;
  }
};
function createBaseIdentifiedConnection(): IdentifiedConnection {
  return {
    id: "",
    client_id: "",
    versions: [],
    state: 0,
    counterparty: Counterparty.fromPartial({}),
    delay_period: BigInt(0)
  };
}
export const IdentifiedConnection = {
  typeUrl: "/ibc.core.connection.v1.IdentifiedConnection",
  encode(message: IdentifiedConnection, writer: BinaryWriter = BinaryWriter.create()): BinaryWriter {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.client_id !== "") {
      writer.uint32(18).string(message.client_id);
    }
    for (const v of message.versions) {
      Version.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    if (message.state !== 0) {
      writer.uint32(32).int32(message.state);
    }
    if (message.counterparty !== undefined) {
      Counterparty.encode(message.counterparty, writer.uint32(42).fork()).ldelim();
    }
    if (message.delay_period !== BigInt(0)) {
      writer.uint32(48).uint64(message.delay_period);
    }
    return writer;
  },
  decode(input: BinaryReader | Uint8Array, length?: number): IdentifiedConnection {
    const reader = input instanceof BinaryReader ? input : new BinaryReader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseIdentifiedConnection();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.client_id = reader.string();
          break;
        case 3:
          message.versions.push(Version.decode(reader, reader.uint32()));
          break;
        case 4:
          message.state = (reader.int32() as any);
          break;
        case 5:
          message.counterparty = Counterparty.decode(reader, reader.uint32());
          break;
        case 6:
          message.delay_period = reader.uint64();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },
  fromJSON(object: any): IdentifiedConnection {
    const obj = createBaseIdentifiedConnection();
    if (isSet(object.id)) obj.id = String(object.id);
    if (isSet(object.client_id)) obj.client_id = String(object.client_id);
    if (Array.isArray(object?.versions)) obj.versions = object.versions.map((e: any) => Version.fromJSON(e));
    if (isSet(object.state)) obj.state = stateFromJSON(object.state);
    if (isSet(object.counterparty)) obj.counterparty = Counterparty.fromJSON(object.counterparty);
    if (isSet(object.delay_period)) obj.delay_period = BigInt(object.delay_period.toString());
    return obj;
  },
  toJSON(message: IdentifiedConnection): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.client_id !== undefined && (obj.client_id = message.client_id);
    if (message.versions) {
      obj.versions = message.versions.map(e => e ? Version.toJSON(e) : undefined);
    } else {
      obj.versions = [];
    }
    message.state !== undefined && (obj.state = stateToJSON(message.state));
    message.counterparty !== undefined && (obj.counterparty = message.counterparty ? Counterparty.toJSON(message.counterparty) : undefined);
    message.delay_period !== undefined && (obj.delay_period = (message.delay_period || BigInt(0)).toString());
    return obj;
  },
  fromPartial<I extends Exact<DeepPartial<IdentifiedConnection>, I>>(object: I): IdentifiedConnection {
    const message = createBaseIdentifiedConnection();
    message.id = object.id ?? "";
    message.client_id = object.client_id ?? "";
    message.versions = object.versions?.map(e => Version.fromPartial(e)) || [];
    message.state = object.state ?? 0;
    if (object.counterparty !== undefined && object.counterparty !== null) {
      message.counterparty = Counterparty.fromPartial(object.counterparty);
    }
    if (object.delay_period !== undefined && object.delay_period !== null) {
      message.delay_period = BigInt(object.delay_period.toString());
    }
    return message;
  }
};
function createBaseCounterparty(): Counterparty {
  return {
    client_id: "",
    connection_id: "",
    prefix: MerklePrefix.fromPartial({})
  };
}
export const Counterparty = {
  typeUrl: "/ibc.core.connection.v1.Counterparty",
  encode(message: Counterparty, writer: BinaryWriter = BinaryWriter.create()): BinaryWriter {
    if (message.client_id !== "") {
      writer.uint32(10).string(message.client_id);
    }
    if (message.connection_id !== "") {
      writer.uint32(18).string(message.connection_id);
    }
    if (message.prefix !== undefined) {
      MerklePrefix.encode(message.prefix, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },
  decode(input: BinaryReader | Uint8Array, length?: number): Counterparty {
    const reader = input instanceof BinaryReader ? input : new BinaryReader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseCounterparty();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.client_id = reader.string();
          break;
        case 2:
          message.connection_id = reader.string();
          break;
        case 3:
          message.prefix = MerklePrefix.decode(reader, reader.uint32());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },
  fromJSON(object: any): Counterparty {
    const obj = createBaseCounterparty();
    if (isSet(object.client_id)) obj.client_id = String(object.client_id);
    if (isSet(object.connection_id)) obj.connection_id = String(object.connection_id);
    if (isSet(object.prefix)) obj.prefix = MerklePrefix.fromJSON(object.prefix);
    return obj;
  },
  toJSON(message: Counterparty): unknown {
    const obj: any = {};
    message.client_id !== undefined && (obj.client_id = message.client_id);
    message.connection_id !== undefined && (obj.connection_id = message.connection_id);
    message.prefix !== undefined && (obj.prefix = message.prefix ? MerklePrefix.toJSON(message.prefix) : undefined);
    return obj;
  },
  fromPartial<I extends Exact<DeepPartial<Counterparty>, I>>(object: I): Counterparty {
    const message = createBaseCounterparty();
    message.client_id = object.client_id ?? "";
    message.connection_id = object.connection_id ?? "";
    if (object.prefix !== undefined && object.prefix !== null) {
      message.prefix = MerklePrefix.fromPartial(object.prefix);
    }
    return message;
  }
};
function createBaseClientPaths(): ClientPaths {
  return {
    paths: []
  };
}
export const ClientPaths = {
  typeUrl: "/ibc.core.connection.v1.ClientPaths",
  encode(message: ClientPaths, writer: BinaryWriter = BinaryWriter.create()): BinaryWriter {
    for (const v of message.paths) {
      writer.uint32(10).string(v!);
    }
    return writer;
  },
  decode(input: BinaryReader | Uint8Array, length?: number): ClientPaths {
    const reader = input instanceof BinaryReader ? input : new BinaryReader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseClientPaths();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.paths.push(reader.string());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },
  fromJSON(object: any): ClientPaths {
    const obj = createBaseClientPaths();
    if (Array.isArray(object?.paths)) obj.paths = object.paths.map((e: any) => String(e));
    return obj;
  },
  toJSON(message: ClientPaths): unknown {
    const obj: any = {};
    if (message.paths) {
      obj.paths = message.paths.map(e => e);
    } else {
      obj.paths = [];
    }
    return obj;
  },
  fromPartial<I extends Exact<DeepPartial<ClientPaths>, I>>(object: I): ClientPaths {
    const message = createBaseClientPaths();
    message.paths = object.paths?.map(e => e) || [];
    return message;
  }
};
function createBaseConnectionPaths(): ConnectionPaths {
  return {
    client_id: "",
    paths: []
  };
}
export const ConnectionPaths = {
  typeUrl: "/ibc.core.connection.v1.ConnectionPaths",
  encode(message: ConnectionPaths, writer: BinaryWriter = BinaryWriter.create()): BinaryWriter {
    if (message.client_id !== "") {
      writer.uint32(10).string(message.client_id);
    }
    for (const v of message.paths) {
      writer.uint32(18).string(v!);
    }
    return writer;
  },
  decode(input: BinaryReader | Uint8Array, length?: number): ConnectionPaths {
    const reader = input instanceof BinaryReader ? input : new BinaryReader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseConnectionPaths();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.client_id = reader.string();
          break;
        case 2:
          message.paths.push(reader.string());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },
  fromJSON(object: any): ConnectionPaths {
    const obj = createBaseConnectionPaths();
    if (isSet(object.client_id)) obj.client_id = String(object.client_id);
    if (Array.isArray(object?.paths)) obj.paths = object.paths.map((e: any) => String(e));
    return obj;
  },
  toJSON(message: ConnectionPaths): unknown {
    const obj: any = {};
    message.client_id !== undefined && (obj.client_id = message.client_id);
    if (message.paths) {
      obj.paths = message.paths.map(e => e);
    } else {
      obj.paths = [];
    }
    return obj;
  },
  fromPartial<I extends Exact<DeepPartial<ConnectionPaths>, I>>(object: I): ConnectionPaths {
    const message = createBaseConnectionPaths();
    message.client_id = object.client_id ?? "";
    message.paths = object.paths?.map(e => e) || [];
    return message;
  }
};
function createBaseVersion(): Version {
  return {
    identifier: "",
    features: []
  };
}
export const Version = {
  typeUrl: "/ibc.core.connection.v1.Version",
  encode(message: Version, writer: BinaryWriter = BinaryWriter.create()): BinaryWriter {
    if (message.identifier !== "") {
      writer.uint32(10).string(message.identifier);
    }
    for (const v of message.features) {
      writer.uint32(18).string(v!);
    }
    return writer;
  },
  decode(input: BinaryReader | Uint8Array, length?: number): Version {
    const reader = input instanceof BinaryReader ? input : new BinaryReader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseVersion();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.identifier = reader.string();
          break;
        case 2:
          message.features.push(reader.string());
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },
  fromJSON(object: any): Version {
    const obj = createBaseVersion();
    if (isSet(object.identifier)) obj.identifier = String(object.identifier);
    if (Array.isArray(object?.features)) obj.features = object.features.map((e: any) => String(e));
    return obj;
  },
  toJSON(message: Version): unknown {
    const obj: any = {};
    message.identifier !== undefined && (obj.identifier = message.identifier);
    if (message.features) {
      obj.features = message.features.map(e => e);
    } else {
      obj.features = [];
    }
    return obj;
  },
  fromPartial<I extends Exact<DeepPartial<Version>, I>>(object: I): Version {
    const message = createBaseVersion();
    message.identifier = object.identifier ?? "";
    message.features = object.features?.map(e => e) || [];
    return message;
  }
};
function createBaseParams(): Params {
  return {
    max_expected_time_per_block: BigInt(0)
  };
}
export const Params = {
  typeUrl: "/ibc.core.connection.v1.Params",
  encode(message: Params, writer: BinaryWriter = BinaryWriter.create()): BinaryWriter {
    if (message.max_expected_time_per_block !== BigInt(0)) {
      writer.uint32(8).uint64(message.max_expected_time_per_block);
    }
    return writer;
  },
  decode(input: BinaryReader | Uint8Array, length?: number): Params {
    const reader = input instanceof BinaryReader ? input : new BinaryReader(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseParams();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.max_expected_time_per_block = reader.uint64();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },
  fromJSON(object: any): Params {
    const obj = createBaseParams();
    if (isSet(object.max_expected_time_per_block)) obj.max_expected_time_per_block = BigInt(object.max_expected_time_per_block.toString());
    return obj;
  },
  toJSON(message: Params): unknown {
    const obj: any = {};
    message.max_expected_time_per_block !== undefined && (obj.max_expected_time_per_block = (message.max_expected_time_per_block || BigInt(0)).toString());
    return obj;
  },
  fromPartial<I extends Exact<DeepPartial<Params>, I>>(object: I): Params {
    const message = createBaseParams();
    if (object.max_expected_time_per_block !== undefined && object.max_expected_time_per_block !== null) {
      message.max_expected_time_per_block = BigInt(object.max_expected_time_per_block.toString());
    }
    return message;
  }
};