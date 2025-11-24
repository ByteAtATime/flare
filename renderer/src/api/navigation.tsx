import {
  createContext,
  Fragment,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import * as protocol from "../protocol";
import { setNavigationPop } from "../index";

declare module "react/jsx-runtime" {
  namespace JSX {
    interface IntrinsicElements {
      "flare-nav-stack": any;
    }
  }
}

export type Route = {
  id: string;
  component: ReactNode;
};

export type Navigation = {
  pop: () => void;
  push: (element: React.ReactElement, onPop?: () => void) => void;
};

const NavigationContext = createContext<Navigation>({
  pop: () => {},
  push: (element, onPop) => {},
});

export const useNavigation = () => useContext(NavigationContext);

export const NavigationRoot: React.FC<{ children: ReactNode }> = (props) => {
  const initialRoute: Route = useMemo(
    () => ({
      id: "route_0",
      component: props.children,
    }),
    [props.children]
  );

  const [routes, setRoutes] = useState<Route[]>([initialRoute]);
  const [routeIdCounter, setRouteIdCounter] = useState(1);

  const push = useCallback(
    (component: ReactNode, onPop?: () => void) => {
      setRoutes((prevRoutes) => {
        const newId = `route_${routeIdCounter}`;
        setRouteIdCounter((c) => c + 1);

        return [
          ...prevRoutes,
          {
            id: newId,
            component,
            onPop,
          },
        ];
      });
    },
    [routeIdCounter]
  );

  const pop = useCallback(() => {
    setRoutes((prevRoutes) => {
      if (prevRoutes.length > 1) {
        return prevRoutes.slice(0, -1);
      }

      protocol.pop();
      return prevRoutes;
    });
  }, []);

  const navigation: Navigation = useMemo(
    () => ({
      push,
      pop,
    }),
    [push, pop]
  );

  useEffect(() => {
    // TODO: better way of handling this?
    setNavigationPop(pop);
  }, [pop]);

  return (
    <NavigationContext.Provider value={navigation}>
      <flare-nav-stack>
        {routes.map((route) => (
          <Fragment key={route.id}>{route.component}</Fragment>
        ))}
      </flare-nav-stack>
    </NavigationContext.Provider>
  );
};
