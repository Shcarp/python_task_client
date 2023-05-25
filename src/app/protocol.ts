
export type Kind = 'request' | 'response' | 'push'

export interface PRequestProps<T> {
    url: string;
    sequence: string;
    data: T;
    sendTime?: number;
}

export enum Status {
    OK = 200,
    BAD_REQUEST = 400,
    NOT_FOUND = 404,
    INTERNAL_SERVER_ERROR = 500
}

    // 任务状态

export class PRequest<T> {
    private _ctype: Kind = 'request';
    private _url: string;
    private _sequence: string;
    private _sendTime: number;
    private _data: T;

    get type() {
        return this._ctype;
    }

    constructor(props: PRequestProps<T>) {
        this._url = props.url;
        this._sequence = props.sequence;
        this._sendTime = props.sendTime ?? Date.now();
        this._data = props.data;
    }

    toJSON() {
        return JSON.stringify({
            url: this._url,
            sequence: this._sequence,
            sendTime: this._sendTime,
            data: this._data,
        });
    }

    static fromJSON<T>(json: string) {
        const { url, type, sequence, sendTime, data } = JSON.parse(json);
        const req = new PRequest<T>({
            url,
            sequence,
            data,
            sendTime
        });
        return req;
    }
}

export interface PResponseProps<T> {
    sequence: string;
    data: T;
    type?: Kind;
    sendTime?: number;
    status: Status;
}

export class PResponse<T> {
    private _ctype: Kind = 'response';

    private _sequence: string
    private _status: Status
    private _sendTime: number
    private _data: T

    get type() {
        return this._ctype;
    }

    get sequence() {
        return this._sequence;
    }

    get status() {
        return this._status;
    }

    get sendTime() {
        return this._sendTime;
    }

    get data() {
        return this._data;
    }

    constructor(props: PResponseProps<T>) {
        this._sequence = props.sequence;
        this._sendTime = props.sendTime ?? Date.now();
        this._data = props.data;
        this._status = props.status;
    }

    toJSON() {
        return JSON.stringify({
            ctype: this._ctype,
            sequence: this._sequence,
            sendTime: this._sendTime,
            data: this._data,
            status: this._status
        });
    }

    static fromJSON<T>(obj: PResponseProps<T>) {
        const { sequence, sendTime, data, status } = obj;
        const res = new PResponse<T>({
            sequence,
            data,
            sendTime,
            status
        });
        return res;
    }
}

export interface PPushProps<T> {
    event: string;
    status: Status;
    data: T;
    type?: Kind;
    sendTime?: number;
}

/**
 * type: str = 'push'
    event: str = None
    status: Status = Status.OK
    sendTime: int = None
    data: any = None
 */

export class PPush<T> {
    private _ctype: Kind = 'push';

    private _event: string
    private _status: Status
    private _sendTime: number
    private _data: T


    get type() {
        return this._ctype;
    }

    get event() {
        return this._event;
    }

    get status() {
        return this._status;
    }

    get sendTime() {
        return this._sendTime;
    }

    get data() {
        return this._data;
    }

    constructor(props: PPushProps<T>) {
        this._event = props.event;
        this._sendTime = props.sendTime ?? Date.now();
        this._data = props.data;
        this._status = props.status;
    }

    toJSON() {
        return JSON.stringify({
            ctype: this._ctype,
            event: this._event,
            sendTime: this._sendTime,
            data: this._data,
            status: this._status
        });
    }

    static fromJSON<T>(obj: PPushProps<T>) {
        const { event, sendTime, data, status } = obj;
        const res = new PPush<T>({
            event,
            data,
            sendTime,
            status
        });
        return res;
    }
}
