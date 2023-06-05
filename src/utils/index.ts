
type EventsDefinition = { [event: string]: (...args: any[]) => void };

type UBAny<T> = {
	[key: string]: T;
};

type UBValueType<T> = T extends UBAny<infer U> ? U : never;

type UnionToIntersection<Union> = (Union extends any ? (argument: Union) => void : never) extends (argument: infer Intersection) => void ? Intersection : never;

export type EventsWithoutAny<T extends { [event: string]: (...args: any[]) => void }> = {
	on: UnionToIntersection<UBValueType<{ [K in keyof T]: (event: K, listener: T[K]) => any }>>;
	emit: UnionToIntersection<UBValueType<{ [K in keyof T]: (event: K, ...args: Parameters<T[K]>) => boolean }>>;
};

export type Events<T extends EventsDefinition> = EventsWithoutAny<T> & {
	on(event: string, listener: (...args: any[]) => void): any;
	emit(event: string, ...args: any[]): any;
};
