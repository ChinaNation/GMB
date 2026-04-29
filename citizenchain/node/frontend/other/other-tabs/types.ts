// 其他 tab 内容类型，对齐后端 src/other/other-tabs。

type OtherIframeTabItem = {
  key: string;
  title: string;
  contentType: 'iframe';
  url: string;
};

type OtherTextTabItem = {
  key: string;
  title: string;
  contentType: 'text';
  text: string;
};

export type OtherTabItem = OtherIframeTabItem | OtherTextTabItem;

export type OtherTabsPayload = {
  tabs: OtherTabItem[];
};
