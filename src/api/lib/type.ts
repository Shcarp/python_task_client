export interface ResponseBase {
    code: number;
    msg: string;
}

export type ResponseData<T> = ResponseBase & {
    data: T;
}
