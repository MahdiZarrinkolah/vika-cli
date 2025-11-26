export type ApiResult<
  SuccessMap extends Record<number, any>,
  ErrorMap extends Record<number, any>
> =
  | {
      ok: true;
      status: keyof SuccessMap;
      data: SuccessMap[keyof SuccessMap];
    }
  | {
      ok: false;
      status: keyof ErrorMap;
      error: ErrorMap[keyof ErrorMap];
    };

